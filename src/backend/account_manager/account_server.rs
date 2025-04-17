use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::aes_keys::decrypted_key::decrypt;
use crate::backend::aes_keys::keys_password::{
    derive_key, generate_random_key, generate_salt_from_login,
};
use crate::backend::file_manager::mapping::init_map;
use crate::backend::{
    VAULTIFY_CONFIG, VAULTIFY_DATABASE, VAULTS_DATA, VAULT_CONFIG_ROOT, VAULT_USERS_DIR,
};
use actix_web::cookie::time::Duration as Dudu; // Replace std::time::Duration with actix_web::cookie::time::Duration
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use base64;
use bcrypt::{hash, verify, DEFAULT_COST};
use dirs;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tera::Context;
use tera::Tera;
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

fn get_user_from_cookie(req: &HttpRequest) -> Option<JWT> {
    req.cookie("user_token")
        .and_then(|cookie| serde_json::from_str(cookie.value()).ok())
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

#[derive(Serialize)]
pub struct CreateUserResponse {
    pub success: bool,
    pub message: String,
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
    pub vault_key: HashMap<String, Vec<u8>>,
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
            vault_key: HashMap::new(),
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

#[derive(Serialize)]
struct Vault {
    id: u32,
    name: String,
    date: i64,
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
pub async fn clean_expired_sessions() {
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
pub async fn create_user_query(form: web::Json<CreateUserForm>) -> HttpResponse {
    let conn = CONNECTION.lock().unwrap();
    let email = form.username.clone();
    let pw = form.password.clone();

    // Check if the user already exists
    if let Ok(Some(_)) = get_user_by_email(&conn, &email) {
        return HttpResponse::Conflict().json(json!({
            "success": false,
            "message": "Un utilisateur avec cet email existe déjà"
        }));
    }

    // Hash the password
    let hash_pw = match hash(&pw, DEFAULT_COST) {
        Ok(hashed) => hashed,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Erreur lors du hachage du mot de passe"
            }))
        }
    };

    // Create the user in the database
    let id = match create_user(&conn, &email, &hash_pw) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Erreur lors de la création de l'utilisateur"
            }))
        }
    };

    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Utilisateur créé avec succès"
    }))
}

/**
 * Endpoint to log in a user.
 *
 * @param form - The form data containing the username and password.
 * @return An HTTP response containing the JWT if the login is successful.
 */

pub async fn login_user_query(form: web::Json<LoginForm>) -> impl Responder {
    let conn = CONNECTION.lock().unwrap();
    let email = form.username.clone();
    let pw = form.password.clone();

    if let Some((user_id, hash_pw)) = get_user_by_email(&conn, &email).unwrap() {
        if verify(&pw, &hash_pw).unwrap() {
            let session_id = generate_session_id();
            let user_key = derive_key(&pw, &generate_salt_from_login(&email), 10000);

            SESSION_CACHE.lock().unwrap().insert(
                session_id.clone(),
                Session::new(user_id, &hash_pw, &user_key),
            );

            let jwt_token = JWT::new(&session_id, user_id, &email);

            // Create the cookie with the JWT
            let cookie = Cookie::build("user_token", serde_json::to_string(&jwt_token).unwrap())
                .http_only(true)
                .secure(true) // Use secure(true) if you are in production (HTTPS)
                .path("/")
                .max_age(Dudu::days(7)) // The cookie expires after 7 days
                .finish();
            // Display the JWT to see if it is generated correctly
            return HttpResponse::Ok().cookie(cookie).json(json!({
                "success": true,
                "message": "Successful connection"
            }));
        }
    }

    HttpResponse::Unauthorized().json(json!({
        "success": false,
        "message": "incorrect Email or Password"
    }))
}

/**
 * Endpoint to get the list of vaults for a user.
 *
 * @param user - The JWT containing the user information.
 * @return An HTTP response containing the list of vaults.
 */
pub async fn get_vaults_list_query(req: HttpRequest) -> HttpResponse {
    if let Some(jwt) = get_user_from_cookie(&req) {
        let conn = CONNECTION.lock().unwrap();
        if let Ok(vaults) = get_user_vaults(&conn, jwt.id) {
            HttpResponse::Ok().json(vaults)
        } else {
            HttpResponse::Unauthorized().finish()
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[derive(Deserialize)]
pub struct VaultForm {
    name: String, // The name must match the `name` attribute of the HTML form
}

/**
 * Creates a new vault for a user.
 *
 * @param conn - The database connection.
 * @param vault_info - The information about the vault.
 * @return A Result containing the ID of the created vault.
 */

pub fn create_vault(conn: &Connection, vault_info: &VaultInfo) -> Result<Vault> {
    conn.execute(
        "INSERT INTO vaults (user_id, name, date) VALUES (?, ?, ?)",
        params![vault_info.user_id, vault_info.name, vault_info.date as i64],
    )?;

    let vault_id = conn.last_insert_rowid() as u32;

    // Construct and return the Vault object
    Ok(Vault {
        id: vault_id,
        name: vault_info.name.clone(),
        date: vault_info.date as i64,
    })
}

/**
 * Endpoint to create a new vault for a user.
 *
 * @param data - A tuple containing the JWT and the name of the vault.
 * @return An HTTP response indicating the result of the operation.
 */
pub async fn create_vault_query(
    req: HttpRequest,
    form: web::Form<VaultForm>, // The form contains only one field (name)
) -> impl Responder {
    if let Some(decoded_jwt) = get_user_from_cookie(&req) {
        let connection = CONNECTION.lock().unwrap();
        let sessions = SESSION_CACHE.lock().unwrap();
        let name: &str = &form.name;
        if let Some(cache) = sessions.get(&decoded_jwt.session_id) {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let vault_name = format!("{}_{}", decoded_jwt.id, time);

            let vault_path = format!("{}/{}{}", ROOT.to_str().unwrap(), VAULTS_DATA, vault_name);

            let vault_config = format!("{}/{}", vault_path, VAULT_CONFIG_ROOT);
            let users_vault = format!("{}/{}", vault_path, VAULT_USERS_DIR);
            let user_json = format!("{users_vault}{}.json", decoded_jwt.id);

            fs::create_dir_all(&vault_path).unwrap();
            fs::create_dir_all(&vault_config).unwrap();
            fs::create_dir_all(&users_vault).unwrap();
            fs::File::create(&user_json).unwrap();

            let info = VaultInfo::new(decoded_jwt.id, &name, time);

            let vault_key = generate_random_key();

            let vault_key = derive_key(
                &String::from_utf8_lossy(vault_key.as_slice()),
                generate_salt_from_login(&decoded_jwt.email).as_slice(),
                10000,
            );

            let content = serde_json::to_string(&(vault_key, Perms::Admin)).unwrap();
            let encrypted_content = encrypt(content.as_bytes(), cache.user_key.as_slice());
            fs::write(&user_json, &encrypted_content).unwrap();

            init_map(
                &format!("{}/map.json", vault_path),
                cache.user_key.as_slice(),
            );

            match create_vault(&connection, &info) {
                Ok(res) => HttpResponse::Ok().json(json!({
                    "message": format!("Coffre '{}' créé avec succès !", res.name),
                    "vault_id": res.id,
                    "date": res.date,
                })),
                Err(e) => {
                    eprintln!("Erreur lors de la création du coffre : {:?}", e);
                    HttpResponse::InternalServerError()
                        .body("Erreur lors de la création du coffre.")
                }
            }
        } else {
            HttpResponse::Unauthorized().body("incorrect email or password")
        }
    } else {
        HttpResponse::Unauthorized().body("Token JWT invalide.")
    }
}

/**
 * Endpoint to load a vault for a user.
 *
 * @param data - A tuple containing the JWT and the VaultInfo.
 * @return An HTTP response containing the updated JWT with the loaded vault.
 */
pub async fn load_vault_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
) -> impl Responder {
    let info = vault_info.into_inner();

    if let Some(mut jwt) = get_user_from_cookie(&req) {
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

            let encrypted_content = match fs::read(&key_path) {
                Ok(data) => data,
                Err(_) => return HttpResponse::NotFound().body("Vault file not found"),
            };

            let decrypted_content =
                match decrypt(encrypted_content.as_slice(), cache.user_key.as_slice()) {
                    Ok(data) => data,
                    Err(_) => return HttpResponse::InternalServerError().body("Failed to decrypt"),
                };

            let (vault_key, vault_perms): (Vec<u8>, Perms) =
                match serde_json::from_str(&String::from_utf8(decrypted_content).unwrap()) {
                    Ok(parsed) => parsed,
                    Err(_) => {
                        return HttpResponse::InternalServerError().body("Invalid vault data")
                    }
                };

            if cache.vault_key.get(&vault_name).is_none() {
                cache.vault_key.insert(vault_name.clone(), vault_key);
                cache.vault_perms = vault_perms;
            }

            jwt.loaded_vault = Some(info.clone());

            HttpResponse::Ok().json(jwt)
        } else {
            HttpResponse::Unauthorized().body("Invalid session")
        }
    } else {
        HttpResponse::InternalServerError().body("Failed to decrypt")
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
