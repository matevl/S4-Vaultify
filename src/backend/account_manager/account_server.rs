use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use lazy_static::lazy_static;
use dirs;
use uuid::Uuid;
use crate::backend::{VAULTIFY_CONFIG, VAULTIFY_DATABASE, VAULTS_DATA};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Perms {
    Admin,
    Write,
    Read,
}

#[derive(serde::Deserialize, Debug)]
pub struct CreateUserForm {
    username: String,
    password: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VaultInfo {
    pub name: String,
    pub path: String,
    pub date: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JWT {
    pub id: i32,
    pub email: String,
    pub loaded_vault: Option<VaultInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JWTPrivate {
    hash_pw: String,
    user_key: Vec<u8>,
    vault_key: Vec<u8>,
    vault_perms: Perms,
}

#[derive(Clone)]
pub struct Session {
    pub user_id: i32,
    pub private_data: JWTPrivate,
    pub last_activity: SystemTime,
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
}

// Function to initialize the connection to the database
pub fn init_db_connection(database_path: &str) -> Result<Connection> {
    Connection::open(database_path)
}

// Function to create a new user
pub fn create_user(conn: &Connection, email: &str, hash_password: &str) -> Result<i32> {
    conn.execute(
        "INSERT INTO users (email, hash_password) VALUES (?, ?)",
        params![email, hash_password],
    )?;
    Ok(conn.last_insert_rowid() as i32)
}

// Function to get a user by email
pub fn get_user_by_email(conn: &Connection, email: &str) -> Result<Option<(i32, String)>> {
    let mut stmt = conn.prepare("SELECT id, hash_password FROM users WHERE email = ?")?;
    let mut rows = stmt.query(params![email])?;
    if let Some(row) = rows.next()? {
        Ok(Some((row.get(0)?, row.get(1)?)))
    } else {
        Ok(None)
    }
}

// Function to create a new vault for a user
pub fn create_vault(conn: &Connection, user_id: i32, name: &str, path: &str, date: u64) -> Result<i32> {
    conn.execute(
        "INSERT INTO vaults (user_id, name, path, date) VALUES (?, ?, ?, ?)",
        params![user_id, name, path, date as i64],
    )?;
    Ok(conn.last_insert_rowid() as i32)
}

// Function to get the vaults of a user
pub fn get_user_vaults(conn: &Connection, user_id: i32) -> Result<Vec<VaultInfo>> {
    let mut stmt = conn.prepare("SELECT name, path, date FROM vaults WHERE user_id = ?")?;
    let mut rows = stmt.query(params![user_id])?;
    let mut vaults = Vec::new();
    while let Some(row) = rows.next()? {
        vaults.push(VaultInfo {
            name: row.get(0)?,
            path: row.get(1)?,
            date: row.get(2)?,
        });
    }
    Ok(vaults)
}

// Function to generate a unique session identifier
fn generate_session_id() -> String {
    Uuid::new_v4().to_string()
}

// Function to clean expired sessions
fn clean_expired_sessions() {
    let mut cache = SESSION_CACHE.lock().unwrap();
    let now = SystemTime::now();
    cache.retain(|_, session| {
        now.duration_since(session.last_activity).unwrap() < Duration::from_secs(3600)
        // 1 hour
    });
}

// Endpoint to create a new user
pub async fn create_user_query(
    conn: web::Data<Connection>,
    form: web::Form<CreateUserForm>,
) -> impl Responder {
    let email = form.username.clone();
    let pw = form.password.clone();
    let hash_pw = hash(pw, DEFAULT_COST).unwrap();
    let user_id = create_user(&conn, &email, &hash_pw).unwrap();
    HttpResponse::Ok().json(JWT {
        id: user_id,
        email: email.clone(),
        loaded_vault: None,
    })
}

// Endpoint to log in a user
pub async fn login_user_query(
    conn: web::Data<Connection>,
    form: web::Form<LoginForm>,
) -> impl Responder {
    let email = form.username.clone();
    let pw = form.password.clone();
    if let Some((user_id, hash_pw)) = get_user_by_email(&conn, &email).unwrap(){
        if verify(&pw, &hash_pw).is_ok() {
            let session_id = generate_session_id();
            let private_data = JWTPrivate {
                hash_pw: hash_pw.to_string(),
                user_key: vec![],  // Replace with the actual user key
                vault_key: vec![], // Replace with the actual vault key
                vault_perms: Perms::Read,
            };
            let session = Session {
                user_id,
                private_data,
                last_activity: SystemTime::now(),
            };
            let mut cache = SESSION_CACHE.lock().unwrap();
            cache.insert(session_id.clone(), session);
            return HttpResponse::Ok().json(JWT {
                id: user_id,
                email: email.clone(),
                loaded_vault: None,
            });
        }
    }
    HttpResponse::NotFound().finish()
}

// Endpoint to get the list of vaults for a user
pub async fn get_vaults_list_query(
    conn: web::Data<Connection>,
    user: web::Json<JWT>,
) -> impl Responder {
    let vaults = get_user_vaults(&conn, user.id).unwrap();
    HttpResponse::Ok().json(vaults)
}

// Endpoint to create a new vault for a user
pub async fn create_vault_query(
    conn: web::Data<Connection>,
    mut jwt: web::Json<JWT>,
    name: web::Json<String>,
) -> impl Responder {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let vault_path = format!(
        "{}/{}{}",
        ROOT.join(VAULTS_DATA).to_str().unwrap(),
        jwt.id,
        time
    );
    fs::create_dir_all(&vault_path).unwrap();
    let vault_id = create_vault(&conn, jwt.id, &name, &vault_path, time).unwrap();
    let info = VaultInfo {
        name: name.into_inner(),
        path: vault_path,
        date: time,
    };
    jwt.loaded_vault = Some(info.clone());
    HttpResponse::Ok().json(jwt)
}

// Endpoint to load a vault for a user
pub async fn load_vault_query(
    conn: web::Data<Connection>,
    mut jwt: web::Json<JWT>,
    info: web::Json<VaultInfo>,
) -> impl Responder {
    // Logic to load a vault
    jwt.loaded_vault = Some(info.clone());
    HttpResponse::Ok().json(jwt)
}

pub async fn init_server_config(root: &PathBuf) {
    // Construct the path for the configuration directory
    let config_path = root.join(VAULTIFY_CONFIG);
    // Create the configuration directory if it doesn't exist
    fs::create_dir_all(&config_path).unwrap_or_else(|why| {
        eprintln!(
            "Error creating the configuration directory: {:?}",
            why
        );
    });
    // Construct the path for the database
    let database_path = root.join(VAULTIFY_DATABASE);
    // Create the directory for the database if it doesn't exist
    if let Some(parent_dir) = database_path.parent() {
        fs::create_dir_all(parent_dir).unwrap_or_else(|why| {
            eprintln!(
                "Error creating the directory for the database: {:?}",
                why
            );
        });
    }
    // Initialize the connection to the database
    let conn = init_db_connection(database_path.to_str().unwrap()).unwrap();
    // Initialize the tables if they don't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL UNIQUE,
            hash_password TEXT NOT NULL
        )",
        [],
    ).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS vaults (
            vault_id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            name TEXT,
            path TEXT,
            date INTEGER,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
        [],
    ).unwrap();
}
