// Import necessary modules from the backend
use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::aes_keys::decrypted_key::decrypt;
use crate::backend::aes_keys::keys_password::{
    derive_key, generate_random_key, generate_salt_from_login,
};
use crate::backend::file_manager::mapping::init_map;
use crate::backend::server_manager::account_manager::{get_user_by_email, Perms, VaultForm};
use crate::backend::server_manager::global_manager::{
    get_user_from_cookie, is_vault_in_cache, CONNECTION, EMAIL_TO_SESSION_KEY, ROOT, SESSION_CACHE,
    VAULTS_CACHE,
};
use crate::backend::{VAULTS_DATA, VAULT_CONFIG_ROOT, VAULT_USERS_DIR};

use actix_web::{test, web, HttpRequest, HttpResponse, Responder};
use rusqlite::ffi::sqlite3_mutex_notheld;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// Relative path to the vault permissions file
const PERMS_PATH: &str = ".vault/perms.json";

// Type alias for the permissions mapping: user ID → permissions
type PermsMap = HashMap<u32, Perms>;

/// Represents metadata for a vault.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VaultInfo {
    pub user_id: u32,
    pub name: String,
    pub date: u64,
}

impl VaultInfo {
    /// Creates a new `VaultInfo` instance.
    pub fn new(user_id: u32, name: &str, date: u64) -> Self {
        Self {
            user_id,
            name: name.to_string(),
            date,
        }
    }

    /// Returns the internal vault name, e.g., "123_1700000000".
    pub fn get_name(&self) -> String {
        format!("{}_{}", self.user_id, self.date)
    }

    /// Constructs the full path to the vault's root directory.
    pub fn get_path(&self) -> String {
        format!(
            "{}/{}{}/.json",
            ROOT.to_str().unwrap(),
            VAULTS_DATA,
            self.get_name(),
        )
    }

    /// Creates the vault directory structure.
    pub async fn create_path(&self) -> Result<(), &str> {
        let vault_path = self.get_path();

        // Create main vault directory
        if let Err(_) = fs::create_dir_all(&vault_path) {
            return Err("cannot create vault");
        }

        // Create vault config directory
        let vault_config = format!("{}{}", vault_path, VAULT_CONFIG_ROOT);
        if let Err(_) = fs::create_dir_all(&vault_config) {
            return Err("cannot create vault");
        }

        // Create user-specific directory inside the vault
        let vault_config_users = format!("{}{}", vault_path, VAULT_USERS_DIR);
        if let Err(_) = fs::create_dir_all(&vault_config_users) {
            return Err("cannot create vault");
        }

        // Create empty permissions file
        let vault_perms = format!("{}{}", vault_path, PERMS_PATH);
        if let Err(_) = fs::File::create(&vault_perms) {
            return Err("cannot create vault");
        }

        Ok(())
    }

    /// Saves the encrypted vault key for a specific user.
    pub async fn save_key(&self, vault_key: &[u8], user_key: &[u8], id: u32) -> Result<(), &str> {
        let key_path = format!("{}{}{}.json", self.get_path(), VAULT_USERS_DIR, id);

        // Create key file if it doesn't exist
        if !std::path::Path::new(&key_path).exists() && fs::File::create(&key_path).is_err() {
            return Err("failed to create file");
        }

        // Serialize the vault key
        let content = match serde_json::to_string(&vault_key.to_vec()) {
            Err(_) => return Err("failed to serialize key"),
            Ok(c) => c,
        };

        // Encrypt the key content with the user key
        let encrypted_content = encrypt(content.as_bytes(), user_key);

        // Write the encrypted content to disk
        match fs::write(&key_path, &encrypted_content) {
            Err(_) => Err("failed to write file"),
            _ => Ok(()),
        }
    }

    /// Sets the encrypted permissions file for the vault.
    pub async fn set_perms(&self, vault_key: &[u8], perms: &PermsMap) -> Result<(), &str> {
        let path = format!("{}{}", self.get_path(), PERMS_PATH);

        // Create file if it doesn't exist
        if !std::path::Path::new(&path).exists() && fs::File::create(&path).is_err() {
            return Err("failed to create file");
        }

        // Serialize and encrypt the permissions
        let content = match serde_json::to_string_pretty(perms) {
            Ok(content) => content,
            Err(_) => return Err("failed to serialize permissions"),
        };

        let encrypted_content = encrypt(content.as_bytes(), vault_key);

        // Write to file
        match fs::write(&path, encrypted_content) {
            Err(_) => Err("failed to write file"),
            Ok(_) => Ok(()),
        }
    }

    /// Retrieves and decrypts the vault's permissions.
    pub async fn get_perms(&self, vault_key: &[u8]) -> Result<PermsMap, &str> {
        let path = format!("{}{}", self.get_path(), PERMS_PATH);

        // Read and decrypt permissions file
        if let Ok(data) = fs::read(path) {
            if let Ok(decrypted) = decrypt(&data, &vault_key) {
                if let Ok(perms) =
                    serde_json::from_str(&String::from_utf8_lossy(decrypted.as_slice()))
                {
                    Ok(perms)
                } else {
                    Err("Failed to decrypt data")
                }
            } else {
                Err("Failed to decrypt data")
            }
        } else {
            Err("Failed to decrypt data")
        }
    }
}

/// Struct representing cached vault data.
pub struct VaultsCache {
    info: VaultInfo,
    perms: PermsMap,
    vault_key: Vec<u8>,
}

impl VaultsCache {
    /// Creates a new `VaultsCache` instance.
    pub fn new(info: &VaultInfo, perms: &PermsMap, vault_key: &Vec<u8>) -> Self {
        VaultsCache {
            info: info.clone(),
            perms: perms.clone(),
            vault_key: vault_key.clone(),
        }
    }
}

/// Inserts a new vault into the database.
pub fn create_vault(conn: &Connection, vault_info: &VaultInfo) -> rusqlite::Result<VaultInfo> {
    if let Ok(_) = conn.execute(
        "INSERT INTO vaults (user_id, name, date) VALUES (?, ?, ?)",
        params![vault_info.user_id, vault_info.name, vault_info.date],
    ) {
        Ok(vault_info.clone())
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

/// HTTP endpoint: creates a new vault for a user.
pub async fn create_vault_query(req: HttpRequest, form: web::Form<VaultForm>) -> impl Responder {
    // Authenticate the user via cookie
    if let Some(decoded_jwt) = get_user_from_cookie(&req) {
        let connection = CONNECTION.lock().unwrap();

        // Check if session exists
        if let Some(session) = SESSION_CACHE.get(&decoded_jwt.session_id) {
            let session = session.lock().unwrap();

            // Get current timestamp
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Create vault metadata
            let info = VaultInfo::new(decoded_jwt.id, &form.name, time);

            // Create vault directories
            if let Err(e) = info.create_path().await {
                eprintln!("Failed to create user JSON file: {:?}", e);
                return HttpResponse::InternalServerError()
                    .body("Failed to create user JSON file.");
            }

            // Generate a random vault key and derive it
            let vault_key = generate_random_key();
            let vault_key = derive_key(
                &String::from_utf8_lossy(vault_key.as_slice()),
                generate_salt_from_login(&decoded_jwt.email).as_slice(),
                10000,
            );

            // Assign creator permissions
            let mut perms = PermsMap::new();
            perms.insert(decoded_jwt.id, Perms::Creator);

            // Save permissions file
            if let Err(_) = info.set_perms(vault_key.as_slice(), &perms).await {
                return HttpResponse::InternalServerError()
                    .body("failed to create user JSON file.");
            }

            // Save encrypted vault key for the user
            if let Err(_) = info
                .save_key(
                    vault_key.as_slice(),
                    session.user_key.as_slice(),
                    session.user_id,
                )
                .await
            {
                return HttpResponse::InternalServerError().body("failed to save key");
            }

            // Initialize vault mapping file
            init_map(
                &format!("{}/map.json", info.get_path()),
                vault_key.as_slice(),
            );

            // Persist the vault in the database
            match create_vault(&connection, &info) {
                Ok(res) => HttpResponse::Ok().json(json!({
                    "message": format!("Vault '{}' created successfully!", res.name),
                    "vault_id": res.user_id.clone(),
                    "date": res.date,
                })),
                Err(e) => {
                    eprintln!("Error creating vault: {:?}", e);
                    HttpResponse::InternalServerError().body("Error creating vault.")
                }
            }
        } else {
            HttpResponse::Unauthorized().body("Invalid email or password")
        }
    } else {
        HttpResponse::Unauthorized().body("Invalid JWT token.")
    }
}

/// HTTP endpoint: loads an existing vault into memory.
pub async fn load_vault_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
) -> impl Responder {
    let info = vault_info.into_inner();

    // Authenticate user
    if let Some(mut jwt) = get_user_from_cookie(&req) {
        // Check if the vault is already cached
        if is_vault_in_cache(&info.get_name()).await {
            jwt.loaded_vault = Some(info.clone());
            HttpResponse::Ok().json(jwt)
        } else if let Some(session) = SESSION_CACHE.get(&jwt.session_id) {
            let mut session = session.lock().unwrap();

            let vault_name = info.get_name();
            let key_path = format!("{}{}{}.json", info.get_path(), VAULT_USERS_DIR, jwt.id);

            // Read user's encrypted key file
            let encrypted_content = match fs::read(&key_path) {
                Ok(data) => data,
                Err(_) => return HttpResponse::NotFound().body("Vault file not found"),
            };

            // Decrypt the vault key
            let decrypted_content =
                match decrypt(encrypted_content.as_slice(), session.user_key.as_slice()) {
                    Ok(data) => data,
                    Err(_) => return HttpResponse::InternalServerError().body("Failed to decrypt"),
                };

            // Parse the key from JSON
            let vault_key: Vec<u8> =
                match serde_json::from_str(&String::from_utf8(decrypted_content).unwrap()) {
                    Ok(parsed) => parsed,
                    Err(_) => {
                        return HttpResponse::InternalServerError().body("Invalid vault data")
                    }
                };

            // Load and decrypt permissions
            let vault_perms: PermsMap = match info.get_perms(&vault_key).await {
                Ok(perms) => perms,
                Err(_) => {
                    return HttpResponse::InternalServerError().body("Invalid vault permissions")
                }
            };

            // Cache the vault in memory
            VAULTS_CACHE.insert(
                vault_name,
                Arc::new(Mutex::new(VaultsCache::new(
                    &info,
                    &vault_perms,
                    &vault_key,
                ))),
            );

            HttpResponse::Ok().json(jwt)
        } else {
            HttpResponse::Unauthorized().body("Invalid session")
        }
    } else {
        HttpResponse::InternalServerError().body("Failed to decrypt")
    }
}

// Stubbed endpoints for future implementation
pub async fn get_perms_query(req: HttpRequest, vault_info: web::Json<VaultInfo>) -> impl Responder {
    load_vault_query(req, web::Json(vault_info.clone())).await;

    let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(cache) => cache,
        None => return HttpResponse::InternalServerError().body("Failed to get vault"),
    };

    let vault = cache.lock().unwrap();

    match vault.perms.get(&vault_info.user_id) {
        Some(perm) => HttpResponse::Ok().json(perm),
        None => HttpResponse::InternalServerError().body("Failed to get vault"),
    }
}

/// Shares a vault with another user by email and sets permissions.
pub async fn share_vault_query(
    req: HttpRequest,
    data: web::Json<(VaultInfo, String, String)>,
) -> impl Responder {
    let _ = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().body("Invalid email or password"),
    };

    let con = CONNECTION.lock().unwrap();
    let (vault_info, email, perm) = data.into_inner();

    // test if other user exist
    let id = match get_user_by_email(&con, &email) {
        Ok(Some((id, _))) => id,
        _ => return HttpResponse::InternalServerError().body("user do not exist"),
    };

    // check if vault could be load or is already_loaded
    if !load_vault_query(req, web::Json(vault_info.clone()))
        .await
        .respond_to(&test::TestRequest::default().to_http_request())
        .status()
        .is_success()
    {
        return HttpResponse::InternalServerError().body("Failed to get vault");
    }

    // case if the other user is connected
    if let Some(key) = EMAIL_TO_SESSION_KEY.get(&email) {
        let user_key = {
            SESSION_CACHE
                .get(&key)
                .unwrap()
                .lock()
                .unwrap()
                .user_key
                .clone()
        };

        // get vault cache
        let vault_cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
            Some(vault_cache) => vault_cache,
            None => return HttpResponse::InternalServerError().body("Failed to get vault"),
        };

        let mut vault = vault_cache.lock().unwrap();
        vault.perms.insert(id, Perms::from_str(&perm));

        let keys = vault.vault_key.clone();
        let perms = vault.perms.clone();

        // save perms of the other user
        if vault_info.set_perms(keys.as_slice(), &perms).await.is_err() {
            return HttpResponse::InternalServerError().body("Failed to set vault");
        }

        // save access of vault_key using the private key of the other user
        if vault_info
            .save_key(keys.as_slice(), user_key.as_slice(), id)
            .await
            .is_err()
        {
            return HttpResponse::InternalServerError().body("Failed to save vault");
        }

        HttpResponse::Ok().json("")
    } else {
        // To handle later
        HttpResponse::InternalServerError().body("Failed to share vault")
    }
}

pub async fn remove_user_from_vault_query(
    req: HttpRequest,
    data: web::Json<(VaultInfo, String)>,
) -> impl Responder {
    HttpResponse::Ok().json("")
}

pub async fn delete_vault_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
) -> impl Responder {
    HttpResponse::Ok().json("")
}

pub async fn leave_vault_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
) -> impl Responder {
    HttpResponse::Ok().json("")
}
