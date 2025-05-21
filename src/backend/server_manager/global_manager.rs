use crate::backend::server_manager::account_manager::{init_db_connection, Session, JWT};
use crate::backend::server_manager::vault_manager::VaultsCache;
use crate::backend::{VAULTIFY_CONFIG, VAULTIFY_DATABASE};
use actix_web::HttpRequest;
use lazy_static::lazy_static;
use moka::sync::Cache;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;

lazy_static! {
    pub static ref EMAIL_TO_SESSION_KEY: Cache<String, String> = {
        Cache::builder().build()
    };

    /**
     * Root directory path for the application.
     */
    pub static ref ROOT: std::path::PathBuf = dirs::home_dir().expect("Could not find home dir");

    /**
     * Global cache for user sessions.
     */
    pub static ref SESSION_CACHE: Cache<String, Arc<Mutex<Session>>> = {
        Cache::builder()
            .time_to_idle(Duration::from_secs(1800))
            .build()
    };

    /**
     * Global cache for vault
     */
    pub static ref VAULTS_CACHE: Cache<String, Arc<Mutex<VaultsCache>>> = {
        Cache::builder()
        .time_to_idle(Duration::from_secs(1800))
        .build()
    };

    /**
     * Global database connection.
     */
    pub static ref CONNECTION: Arc<Mutex<Connection>> = Arc::new(Mutex::new(init_db_connection(&format!("{}/{}", ROOT.to_str().unwrap(), VAULTIFY_DATABASE)).unwrap()));
}

pub fn get_user_from_cookie(req: &HttpRequest) -> Option<JWT> {
    req.cookie("user_token")
        .and_then(|cookie| serde_json::from_str(cookie.value()).ok())
}

pub async fn is_vault_in_cache(name: &str) -> bool {
    VAULTS_CACHE.contains_key(name)
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
            id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            date INTEGER NOT NULL,
            FOREIGN KEY (id) REFERENCES users(id)
            PRIMARY KEY (id, user_id, date)
        )",
        [],
    )
    .unwrap();
}
