use crate::backend::aes_keys::keys_password::{derive_key, generate_salt_from_login};
use crate::backend::server_manager::global_manager::{
    get_user_from_cookie, CONNECTION, EMAIL_TO_SESSION_KEY, PENDING_SHARE_CACHE, ROOT,
    SESSION_CACHE,
};
use crate::backend::server_manager::vault_manager::{create_vault, VaultInfo};
use crate::backend::{VAULTS_DATA, VAULT_USERS_DIR};
use actix_web::cookie::time::{Duration as Dudu, Duration, OffsetDateTime};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use uuid::Uuid;

/**
 * Enum representing user permissions.
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Perms {
    NoLoad,
    Read,
    Write,
    Admin,
    Creator,
}

impl Perms {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Creator" => Self::Creator,
            "Admin" => Self::Admin,
            "Write" => Self::Write,
            "Read" => Self::Read,
            "NoLoad" => Self::NoLoad,
            _ => Self::NoLoad,
        }
    }
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
            last_activity: SystemTime::now(),
        }
    }
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
 * Retrieves the vaults of a user from the database.
 *
 * @param conn - The database connection.
 * @param user_id - The ID of the user.
 * @return A Result containing a vector of VaultInfo.
 */
pub fn get_user_vaults(conn: &Connection, user_id: u32) -> Result<Vec<VaultInfo>> {
    let mut stmt = conn.prepare(
        "SELECT id, creator_id, name, date
         FROM vaults
         WHERE id = ?",
    )?;
    let mut rows = stmt.query(params![user_id])?;
    let mut vaults = Vec::new();

    while let Some(row) = rows.next()? {
        vaults.push(VaultInfo {
            creator_id: row.get(1)?,
            name: row.get(2)?,
            date: row.get(3)?,
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
    let _ = match create_user(&conn, &email, &hash_pw) {
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

    let (user_id, hash_pw) = match get_user_by_email(&conn, &email).unwrap() {
        Some((user_id, hash_pw)) => (user_id, hash_pw),
        None => return HttpResponse::Unauthorized().json("invalid email or password"),
    };

    if !verify(&pw, &hash_pw).unwrap() {
        return HttpResponse::Unauthorized().json("invalid email or password");
    }

    let session_key = match EMAIL_TO_SESSION_KEY.get(&email) {
        Some(session_key) => session_key,
        None => {
            let session_id = generate_session_id();
            let user_key = derive_key(&pw, &generate_salt_from_login(&email), 10000);

            SESSION_CACHE.insert(
                session_id.clone(),
                Arc::new(Mutex::new(Session::new(user_id, &hash_pw, &user_key))),
            );

            EMAIL_TO_SESSION_KEY.insert(email.clone(), session_id.clone());

            if let Some(pending) = PENDING_SHARE_CACHE.get(&email) {
                let mut pending = pending.lock().unwrap();
                for (vault_info, vault_key) in pending.drain(..) {
                    vault_info
                        .save_key(vault_key.as_slice(), user_key.as_slice(), user_id)
                        .unwrap();
                    create_vault(&conn, &vault_info, user_id).unwrap();
                }
            }

            session_id
        }
    };
    let jwt = JWT::new(&session_key, user_id, &email);

    let cookie = Cookie::build("user_token", serde_json::to_string(&jwt).unwrap())
        .http_only(true)
        .secure(true) // Use secure(true) if you are in production (HTTPS)
        .path("/")
        .max_age(Dudu::days(7)) // The cookie expires after 7 days
        .finish();

    // Display the JWT to see if it is generated correctly
    HttpResponse::Ok().cookie(cookie).json(json!({
        "success": true,
        "message": "Successful connection"
    }))
}

pub async fn logout_user_query(req: HttpRequest) -> impl Responder {
    let expired_cookie = Cookie::build("user_token", "")
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .expires(OffsetDateTime::now_utc() - Duration::days(1)) // important
        .max_age(Duration::seconds(0)) // important aussi
        .finish();

    HttpResponse::Ok()
        .cookie(expired_cookie)
        .json(json!({ "success": true }))
}

#[derive(Deserialize)]
pub struct VaultForm {
    pub(crate) name: String, // The name must match the `name` attribute of the HTML form
}
