use crate::backend::server_manager::account_manager::{init_db_connection, JWT, ROOT};
use crate::backend::{VAULTIFY_CONFIG, VAULTIFY_DATABASE};
use actix_web::HttpRequest;
use std::fs;

pub fn get_user_from_cookie(req: &HttpRequest) -> Option<JWT> {
    req.cookie("user_token")
        .and_then(|cookie| serde_json::from_str(cookie.value()).ok())
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
