use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::aes_keys::decrypted_key::decrypt;
use crate::backend::aes_keys::keys_password::{
    derive_key, generate_random_key, generate_salt_from_login,
};
use crate::backend::{
    VAULTIFY_CONFIG, VAULTIFY_DATABASE, VAULTS_DATA, VAULT_CONFIG_ROOT, VAULT_USERS_DIR,
};
use actix_web::{web, HttpResponse, Responder};
use base64;
use bcrypt::{hash, verify, DEFAULT_COST};
use dirs;
use lazy_static::lazy_static;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/**
 * Enum representing user permissions.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Perms {
    Admin,
    Write,
    Read,
    NoLoad,
}

/**
 * Struct representing the form data for creating a new user.
 */
#[derive(serde::Deserialize, Debug)]
pub struct CreateUserForm {
    username: String,
    password: String,
}

/**
 * Struct representing the form data for logging in a user.
 */
#[derive(serde::Deserialize, Debug)]
pub struct LoginForm {
    username: String,
    password: String,
}

/**
 * Struct representing information about a vault.
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VaultInfo {
    pub user_id: u32,
    pub name: String,
    pub date: u64,
}

impl VaultInfo {
    /**
     * Creates a new VaultInfo instance.
     *
     * @param user_id - The ID of the user.
     * @param name - The name of the vault.
     * @param date - The creation date of the vault.
     * @return A new VaultInfo instance.
     */
    pub fn new(user_id: u32, name: &str, date: u64) -> Self {
        Self {
            user_id,
            name: name.to_string(),
            date,
        }
    }
}

/**
 * Struct representing a JSON Web Token (JWT).
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JWT {
    pub session_id: String,
    pub id: u32,
    pub email: String,
    pub loaded_vault: Option<VaultInfo>,
}

impl JWT {
    /**
     * Creates a new JWT instance.
     *
     * @param session_id - The session ID.
     * @param id - The user ID.
     * @param email - The user's email.
     * @return A new JWT instance.
     */
    pub fn new(session_id: &str, id: u32, email: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            id,
            email: email.to_string(),
            loaded_vault: None,
        }
    }
}

/**
 * Struct representing a user session.
 */
#[derive(Clone)]
pub struct Session {
    pub user_id: u32,
    pub hash_pw: String,
    pub user_key: Vec<u8>,
    pub vault_key: Vec<u8>,
    pub vault_perms: Perms,
    pub last_activity: SystemTime,
}

impl Session {
    /**
     * Creates a new Session instance.
     *
     * @param user_id - The ID of the user.
     * @param hash_pw - The hashed password of the user.
     * @param user_key - The user's encryption key.
     * @return A new Session instance.
     */
    pub fn new(user_id: u32, hash_pw: &str, user_key: &[u8]) -> Self {
        Self {
            user_id,
            hash_pw: hash_pw.to_string(),
            user_key: user_key.to_vec(),
            vault_key: vec![],
            vault_perms: Perms::NoLoad,
            last_activity: SystemTime::now(),
        }
    }
}

lazy_static! {
    /**
     * Root directory path for the application.
     */
    pub static ref ROOT: std::path::PathBuf = dirs::home_dir().expect("Could not find home dir");

    /**
     * Global cache for user sessions.
     */
    pub static ref SESSION_CACHE: Arc<Mutex<HashMap<String, Session>>> = Arc::new(Mutex::new(HashMap::new()));

    /**
     * Global database connection.
     */
    pub static ref CONNECTION: Arc<Mutex<Connection>> = Arc::new(Mutex::new(init_db_connection(&format!("{}/{}", ROOT.to_str().unwrap(), VAULTIFY_DATABASE)).unwrap()));
}

/**
 * Initializes the connection to the database.
 *
 * @param database_path - The path to the database file.
 * @return A Result containing the Connection object.
 */
pub fn init_db_connection(database_path: &str) -> Result<Connection> {
    Connection::open(database_path)
}

/**
 * Creates a new user in the database.
 *
 * @param conn - The database connection.
 * @param email - The user's email.
 * @param hash_password - The hashed password of the user.
 * @return A Result containing the ID of the created user.
 */
pub fn create_user(conn: &Connection, email: &str, hash_password: &str) -> Result<u32> {
    conn.execute(
        "INSERT INTO users (email, hash_password) VALUES (?, ?)",
        params![email, hash_password],
    )?;
    Ok(conn.last_insert_rowid() as u32)
}

/**
 * Retrieves a user from the database by email.
 *
 * @param conn - The database connection.
 * @param email - The user's email.
 * @return A Result containing an Option with the user's ID and hashed password.
 */
pub fn get_user_by_email(conn: &Connection, email: &str) -> Result<Option<(u32, String)>> {
    let mut stmt = conn.prepare("SELECT id, hash_password FROM users WHERE email = ?")?;
    let mut rows = stmt.query(params![email])?;
    if let Some(row) = rows.next()? {
        Ok(Some((row.get(0)?, row.get(1)?)))
    } else {
        Ok(None)
    }
}

/**
 * Creates a new vault for a user.
 *
 * @param conn - The database connection.
 * @param vault_info - The information about the vault.
 * @return A Result containing the ID of the created vault.
 */
pub fn create_vault(conn: &Connection, vault_info: &VaultInfo) -> Result<u32> {
    conn.execute(
        "INSERT INTO vaults (user_id, name, date) VALUES (?, ?, ?)",
        params![vault_info.user_id, vault_info.name, vault_info.date as i64],
    )?;
    Ok(conn.last_insert_rowid() as u32)
}

/**
 * Retrieves the vaults of a user from the database.
 *
 * @param conn - The database connection.
 * @param user_id - The ID of the user.
 * @return A Result containing a vector of VaultInfo.
 */
pub fn get_user_vaults(conn: &Connection, user_id: u32) -> Result<Vec<VaultInfo>> {
    let mut stmt = conn.prepare(
        "SELECT user_id, name, date
         FROM vaults
         WHERE user_id = ?",
    )?;
    let mut rows = stmt.query(params![user_id])?;
    let mut vaults = Vec::new();

    while let Some(row) = rows.next()? {
        vaults.push(VaultInfo {
            user_id: row.get(0)?,
            name: row.get(1)?,
            date: row.get(2)?,
        });
    }

    Ok(vaults)
}

/**
 * Generates a unique session identifier.
 *
 * @return A String representing the session ID.
 */
fn generate_session_id() -> String {
    Uuid::new_v4().to_string()
}

/**
 * Cleans expired sessions from the cache.
 */
fn clean_expired_sessions() {
    let mut cache = SESSION_CACHE.lock().unwrap();
    let now = SystemTime::now();
    cache.retain(|_, session| {
        now.duration_since(session.last_activity).unwrap() < Duration::from_secs(3600)
        // 1 hour
    });
}

/**
 * Endpoint to create a new user.
 *
 * @param form - The form data containing the username and password.
 * @return An HTTP response indicating the result of the operation.
 */
pub async fn create_user_query(form: web::Form<CreateUserForm>) -> impl Responder {
    let conn = CONNECTION.lock().unwrap();
    let email = form.username.clone();
    let pw = form.password.clone();

    // Check if the user already exists
    if let Ok(Some(_)) = get_user_by_email(&conn, &email) {
        return HttpResponse::Conflict().body("User with this email already exists");
    }

    let hash_pw = match hash(&pw, DEFAULT_COST) {
        Ok(hashed) => hashed,
        Err(_) => return HttpResponse::InternalServerError().body("Error hashing password"),
    };

    let _ = match create_user(&conn, &email, &hash_pw) {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().body("Error creating user"),
    };

    HttpResponse::Ok().json("User created successfully")
}

/**
 * Endpoint to log in a user.
 *
 * @param form - The form data containing the username and password.
 * @return An HTTP response containing the JWT if the login is successful.
 */
pub async fn login_user_query(form: web::Form<LoginForm>) -> impl Responder {
    let conn = CONNECTION.lock().unwrap();
    let email = form.username.clone();
    let pw = form.password.clone();
    if let Some((user_id, hash_pw)) = get_user_by_email(&conn, &email).unwrap() {
        if verify(&pw, &hash_pw).unwrap() {
            let user_key = derive_key(&pw, &generate_salt_from_login(&email), 10000);
            let session_id = generate_session_id();
            SESSION_CACHE.lock().unwrap().insert(
                session_id.clone(),
                Session::new(user_id, &hash_pw, &user_key),
            );
            return HttpResponse::Ok().json(JWT::new(&session_id, user_id, &email));
        }
    }
    HttpResponse::Unauthorized().finish()
}

/**
 * Endpoint to get the list of vaults for a user.
 *
 * @param user - The JWT containing the user information.
 * @return An HTTP response containing the list of vaults.
 */
pub async fn get_vaults_list_query(user: web::Json<JWT>) -> impl Responder {
    let conn = CONNECTION.lock().unwrap();
    if let Ok(vaults) = get_user_vaults(&conn, user.id) {
        HttpResponse::Ok().json(vaults)
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

/**
 * Endpoint to create a new vault for a user.
 *
 * @param data - A tuple containing the JWT and the name of the vault.
 * @return An HTTP response indicating the result of the operation.
 */
pub async fn create_vault_query(data: web::Json<(JWT, String)>) -> impl Responder {
    let connection = CONNECTION.lock().unwrap();
    let sessions = SESSION_CACHE.lock().unwrap();

    let (jwt, name) = data.into_inner();

    if let Some(cache) = sessions.get(&jwt.session_id) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let vault_name = format!("{}_{}", jwt.id, time);

        let vault_path = format!("{}/{}{}", ROOT.to_str().unwrap(), VAULTS_DATA, vault_name);

        let vault_config = format!("{}/{}", vault_path, VAULT_CONFIG_ROOT);
        let users_vault = format!("{}/{}", vault_path, VAULT_USERS_DIR);
        let user_json = format!("{users_vault}{}.json", jwt.id);

        fs::create_dir_all(&vault_path).unwrap();
        fs::create_dir_all(&vault_config).unwrap();
        fs::create_dir_all(&users_vault).unwrap();
        fs::File::create(&user_json).unwrap();

        let info = VaultInfo::new(jwt.id, &name, time);
        create_vault(&connection, &info).unwrap();

        let vault_key = generate_random_key();

        let vault_key = derive_key(
            &base64::encode(&vault_key),
            generate_salt_from_login(&jwt.email).as_slice(),
            10000,
        );

        let content = serde_json::to_string(&(vault_key, Perms::Admin)).unwrap();
        let encrypted_content = encrypt(content.as_bytes(), cache.user_key.as_slice());
        fs::write(&user_json, &encrypted_content).unwrap();
        HttpResponse::Ok().json(format!("vault {} created successfully", name))
    } else {
        HttpResponse::ExpectationFailed().finish()
    }
}

/**
 * Endpoint to load a vault for a user.
 *
 * @param data - A tuple containing the JWT and the VaultInfo.
 * @return An HTTP response containing the updated JWT with the loaded vault.
 */
pub async fn load_vault_query(data: web::Json<(JWT, VaultInfo)>) -> impl Responder {
    let (mut jwt, info) = data.into_inner();

    if let Some(cache) = SESSION_CACHE.lock().unwrap().get_mut(&jwt.session_id) {
        let vault_name = format!("{}_{}", info.user_id, info.date);

        let key_path = format!(
            "{}/{}{}/{}{}.json",
            ROOT.to_str().unwrap(),
            VAULTS_DATA,
            vault_name,
            VAULT_USERS_DIR,
            jwt.id
        );

        let encrypted_content = fs::read(&key_path).unwrap();
        let decrypted_content =
            decrypt(encrypted_content.as_slice(), cache.user_key.as_slice()).unwrap();
        let (vault_key, vault_perms): (Vec<u8>, Perms) =
            serde_json::from_str(&String::from_utf8(decrypted_content).unwrap()).unwrap();
        cache.vault_key = vault_key;
        cache.vault_perms = vault_perms;

        jwt.loaded_vault = Some(info.clone());
        HttpResponse::Ok().json(jwt)
    } else {
        HttpResponse::ExpectationFailed().finish()
    }
}

/**
 * Initializes the server configuration.
 */
pub fn init_server_config() {
    let config_path = ROOT.join(VAULTIFY_CONFIG);
    fs::create_dir_all(&config_path).unwrap_or_else(|why| {
        eprintln!("Error creating the configuration directory: {:?}", why);
    });

    let database_path = ROOT.join(VAULTIFY_DATABASE);
    if let Some(parent_dir) = database_path.parent() {
        fs::create_dir_all(parent_dir).unwrap_or_else(|why| {
            eprintln!("Error creating the directory for the database: {:?}", why);
        });
    }

    let conn = init_db_connection(database_path.to_str().unwrap()).unwrap();

    // Create the users table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL UNIQUE,
            hash_password TEXT NOT NULL
        )",
        [],
    )
    .unwrap();

    // Create the vaults table with a foreign key to users
    conn.execute(
        "CREATE TABLE IF NOT EXISTS vaults (
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            date INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
        [],
    )
    .unwrap();
}
