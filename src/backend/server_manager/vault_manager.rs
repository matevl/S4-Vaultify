// Import necessary modules from the backend
use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::aes_keys::decrypted_key::decrypt;
use crate::backend::aes_keys::keys_password::{
    derive_key, generate_random_key, generate_salt_from_login,
};
use crate::backend::server_manager::account_manager::{get_user_by_email, Perms, VaultForm, JWT};
use crate::backend::server_manager::file_manager::file_tree::{Directory, FILE_TREE_FILE_NAME};
use crate::backend::server_manager::global_manager::{
    get_user_from_cookie, is_vault_in_cache, CONNECTION, EMAIL_TO_SESSION_KEY, PENDING_SHARE_CACHE,
    ROOT, SESSION_CACHE, VAULTS_CACHE,
};
use crate::backend::{VAULTS_DATA, VAULT_CONFIG_ROOT, VAULT_USERS_DIR};

use actix_web::{test, web, HttpRequest, HttpResponse, Responder};
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
    pub creator_id: u32,
    pub name: String,
    pub date: u64,
}

impl VaultInfo {
    /// Creates a new `VaultInfo` instance.
    pub fn new(creator_id: u32, name: &str, date: u64) -> Self {
        Self {
            creator_id,
            name: name.to_string(),
            date,
        }
    }

    /// Returns the internal vault name, e.g., "123_1700000000".
    pub fn get_name(&self) -> String {
        format!("{}_{}", self.creator_id, self.date)
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
    pub fn create_path(&self) -> Result<(), &str> {
        let vault_path = self.get_path();

        // Create main vault directory
        if fs::create_dir_all(&vault_path).is_err() {
            return Err("cannot create vault");
        }

        // Create vault config directory
        let vault_config = format!("{}{}", vault_path, VAULT_CONFIG_ROOT);
        if fs::create_dir_all(&vault_config).is_err() {
            return Err("cannot create vault");
        }

        // Create user-specific directory inside the vault
        let vault_config_users = format!("{}{}", vault_path, VAULT_USERS_DIR);
        if fs::create_dir_all(&vault_config_users).is_err() {
            return Err("cannot create vault");
        }

        // Create empty permissions file
        let vault_perms = format!("{}{}", vault_path, PERMS_PATH);
        if fs::File::create(&vault_perms).is_err() {
            return Err("cannot create vault");
        }

        let file_tree = format!("{}{}", vault_path, FILE_TREE_FILE_NAME);
        if fs::File::create(&file_tree).is_err() {
            return Err("cannot create vault");
        }
        Ok(())
    }
    /// get key path
    pub fn get_key_path(&self, id: u32) -> String {
        format!("{}{}{}.json", self.get_path(), VAULT_USERS_DIR, id)
    }
    /// Saves the encrypted vault key for a specific user.
    pub fn save_key(&self, vault_key: &[u8], user_key: &[u8], id: u32) -> Result<(), &str> {
        let key_path = self.get_key_path(id);

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
    pub fn set_perms(&self, vault_key: &[u8], perms: &PermsMap) -> Result<(), &str> {
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
    pub fn get_perms(&self, vault_key: &[u8]) -> Result<PermsMap, &str> {
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

    /// save file tree
    pub fn save_file_tree(&self, vault_key: &[u8], file_map: Directory) -> Result<(), &str> {
        let content = match serde_json::to_string_pretty(&file_map) {
            Ok(content) => content,
            Err(_) => return Err("failed to serialize permissions"),
        };

        let encrypted_content = encrypt(content.as_bytes(), vault_key);
        if fs::write(
            format!("{}{}", self.get_path(), FILE_TREE_FILE_NAME),
            encrypted_content,
        )
        .is_err()
        {
            return Err("failed to write file");
        }
        Ok(())
    }

    /// get file tree
    pub fn get_file_tree(&self, vault_key: &[u8]) -> Result<Directory, &str> {
        let path = format!("{}{}", self.get_path(), FILE_TREE_FILE_NAME);
        let content = match fs::read(path) {
            Ok(data) => data,
            Err(_) => return Err("failed to read file"),
        };
        let decrypted_content = match decrypt(&content, vault_key) {
            Ok(data) => match String::from_utf8(data) {
                Ok(content) => content,
                Err(_) => return Err("failed to deserialize data"),
            },
            Err(_) => return Err("failed to decrypt data"),
        };

        match serde_json::from_str(&decrypted_content) {
            Ok(dir) => Ok(dir),
            Err(_) => Err("failed to deserialize data"),
        }
    }
}

/// Struct representing cached vault data.
pub struct VaultsCache {
    pub info: VaultInfo,
    pub perms: PermsMap,
    pub vault_key: Vec<u8>,
    pub vault_file_tree: Directory,
}

impl VaultsCache {
    /// Creates a new `VaultsCache` instance.
    pub fn new(
        info: &VaultInfo,
        perms: &PermsMap,
        vault_key: &Vec<u8>,
        file_tree: &Directory,
    ) -> Self {
        VaultsCache {
            info: info.clone(),
            perms: perms.clone(),
            vault_key: vault_key.clone(),
            vault_file_tree: file_tree.clone(),
        }
    }
}

/// Inserts a new vault into the database.
pub fn create_vault(
    conn: &Connection,
    vault_info: &VaultInfo,
    id: u32,
) -> rusqlite::Result<VaultInfo> {
    if let Ok(_) = conn.execute(
        "INSERT INTO vaults (id, creator_id, name, date) VALUES (?, ?, ?, ?)",
        params![id, vault_info.creator_id, vault_info.name, vault_info.date],
    ) {
        Ok(vault_info.clone())
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

pub fn remove_vault(conn: &Connection, vault_info: &VaultInfo, id: u32) -> rusqlite::Result<()> {
    let rows_affected = conn.execute(
        "DELETE FROM vaults WHERE (id, creator_id, name, date) VALUES (?, ?, ?, ?)",
        params![id, vault_info.creator_id, vault_info.name, vault_info.date],
    )?;

    if rows_affected == 1 {
        Ok(())
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows)
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
            if let Err(e) = info.create_path() {
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
            if let Err(_) = info.set_perms(vault_key.as_slice(), &perms) {
                return HttpResponse::InternalServerError()
                    .body("failed to create user JSON file.");
            }

            // Save encrypted vault key for the user
            if info
                .save_key(
                    vault_key.as_slice(),
                    session.user_key.as_slice(),
                    session.user_id,
                )
                .is_err()
            {
                return HttpResponse::InternalServerError().body("failed to save key");
            }

            if info
                .save_file_tree(&vault_key, Directory::new("root".to_string()))
                .is_err()
            {
                return HttpResponse::InternalServerError().body("failed to save file tree");
            }
            // Persist the vault in the database
            match create_vault(&connection, &info, session.user_id) {
                Ok(res) => HttpResponse::Ok().json(json!({
                    "message": format!("Vault '{}' created successfully!", res.name),
                    "vault_id": res.creator_id.clone(),
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
    match load_vault(req, vault_info).await {
        Ok(jwt) => HttpResponse::Ok().json(jwt),
        Err(_) => HttpResponse::InternalServerError().body("failed_to_create_vault"),
    }
}

/// loads an existing vault into memory.
pub async fn load_vault(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
) -> Result<JWT, &'static str> {
    let info = vault_info.into_inner();

    // Authenticate user
    if let Some(mut jwt) = get_user_from_cookie(&req) {
        // Check if the vault is already cached
        if is_vault_in_cache(&info.get_name()).await {
            jwt.loaded_vault = Some(info.clone());
            Ok(jwt)
        } else if let Some(session) = SESSION_CACHE.get(&jwt.session_id) {
            let session = session.lock().unwrap();

            let vault_name = info.get_name();
            let key_path = info.get_key_path(jwt.id);

            // Read user's encrypted key file
            let encrypted_content = match fs::read(&key_path) {
                Ok(data) => data,
                Err(_) => return Err("Vault file not found"),
            };

            // Decrypt the vault key
            let decrypted_content =
                match decrypt(encrypted_content.as_slice(), session.user_key.as_slice()) {
                    Ok(data) => data,
                    Err(_) => return Err("Failed to decrypt"),
                };

            // Parse the key from JSON
            let vault_key: Vec<u8> =
                match serde_json::from_str(&String::from_utf8(decrypted_content).unwrap()) {
                    Ok(parsed) => parsed,
                    Err(_) => return Err("Invalid vault data"),
                };
            // Load and decrypt permissions
            let vault_perms: PermsMap = match info.get_perms(&vault_key) {
                Ok(perms) => perms,
                Err(_) => return Err("Invalid vault permissions"),
            };

            let vault_file_tree = match info.get_file_tree(&vault_key) {
                Ok(file_tree) => file_tree,
                Err(_) => return Err("Invalid vault file tree"),
            };

            // Cache the vault in memory
            VAULTS_CACHE.insert(
                vault_name,
                Arc::new(Mutex::new(VaultsCache::new(
                    &info,
                    &vault_perms,
                    &vault_key,
                    &vault_file_tree,
                ))),
            );

            Ok(jwt)
        } else {
            Err("Invalid session")
        }
    } else {
        Err("Failed to decrypt")
    }
}

// Stubbed endpoints for future implementation
pub async fn get_perms_query(req: HttpRequest, vault_info: web::Json<VaultInfo>) -> impl Responder {
    let jwt = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().body("Invalid email or password"),
    };

    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().body("Failed to get vault");
    }

    let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(cache) => cache,
        None => return HttpResponse::InternalServerError().body("Failed to get vault"),
    };

    let vault = cache.lock().unwrap();

    match vault.perms.get(&jwt.id) {
        Some(perm) => HttpResponse::Ok().json(perm),
        None => HttpResponse::InternalServerError().body("Failed to get vault"),
    }
}

/// Shares a vault with another user by email and sets permissions.
pub async fn share_vault_query(
    req: HttpRequest,
    data: web::Json<(VaultInfo, String, String)>,
) -> impl Responder {
    let jwt = match get_user_from_cookie(&req) {
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
    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().body("Failed to get vault");
    }

    // get vault cache
    let vault_cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(vault_cache) => vault_cache,
        None => return HttpResponse::InternalServerError().body("Failed to get vault"),
    };

    let mut vault = vault_cache.lock().unwrap();

    if !vault.perms.contains_key(&jwt.id) || vault.perms.get(&jwt.id).unwrap() < &Perms::Admin {
        return HttpResponse::Unauthorized()
            .body("You do not have permission to delete this vault");
    }

    vault.perms.insert(id, Perms::from_str(&perm));

    let keys = vault.vault_key.clone();
    let perms = vault.perms.clone();

    // save perms of the other user
    if vault_info.set_perms(keys.as_slice(), &perms).is_err() {
        return HttpResponse::InternalServerError().body("Failed to set vault");
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

        // save access of vault_key using the private key of the other user
        if vault_info
            .save_key(keys.as_slice(), user_key.as_slice(), id)
            .is_err()
        {
            return HttpResponse::InternalServerError().body("Failed to save vault");
        }

        // Persist the vault in the database
        match create_vault(&con, &vault_info, id) {
            Ok(res) => HttpResponse::Ok().json(json!({
                "message": format!("Vault '{}' shared successfully!", res.name),
                "vault_id": res.creator_id.clone(),
                "date": res.date,
            })),
            Err(_) => return HttpResponse::InternalServerError().body("Error creating vault."),
        };

        HttpResponse::Ok().json("")
    } else {
        let to_share = (vault_info.clone(), keys);
        if let Some(pending) = PENDING_SHARE_CACHE.get(&email) {
            let mut pending = pending.lock().unwrap();
            pending.push(to_share);
        } else {
            PENDING_SHARE_CACHE.insert(email, Arc::new(Mutex::new(vec![to_share])));
        }
        HttpResponse::Ok().json("")
    }
}

pub async fn remove_user_from_vault_query(
    req: HttpRequest,
    data: web::Json<(VaultInfo, String)>,
) -> impl Responder {
    let jwt = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Unauthorized().body("Invalid email or password"),
    };

    let (vault_info, email_to_remove) = data.into_inner();

    if load_vault(req, web::Json(vault_info.clone()))
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().body("Failed to get vault");
    }

    let con = CONNECTION.lock().unwrap();
    let id_to_remove = match get_user_by_email(&con, &email_to_remove) {
        Ok(Some((id, _))) => id,
        _ => return HttpResponse::InternalServerError().body("user do not exist"),
    };

    let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
        Some(cache) => cache,
        None => return HttpResponse::InternalServerError().body("Failed to get vault"),
    };

    let mut vault = cache.lock().unwrap();
    {
        let perms = &mut vault.perms;

        let (p1, p2) = match (perms.get(&jwt.id), perms.get(&id_to_remove)) {
            (Some(p1), Some(p2)) => (p1, p2),
            _ => return HttpResponse::InternalServerError().body("Failed to get vault"),
        };

        if p1 < p2 {
            return HttpResponse::InternalServerError().body("Failed to get vault");
        }
        perms.remove(&id_to_remove);
    }
    let key = vault.vault_key.as_slice();
    if vault_info.set_perms(key, &vault.perms).is_err() {
        return HttpResponse::InternalServerError().body("Failed to set vault");
    }

    if fs::remove_file(vault_info.get_key_path(id_to_remove)).is_err() {
        return HttpResponse::InternalServerError().body("Failed to remove file");
    }

    match remove_vault(&con, &vault_info, id_to_remove) {
        Ok(_) => HttpResponse::Ok().json(""),
        Err(_) => HttpResponse::InternalServerError().body("Failed to remove vault"),
    }
}

pub async fn delete_vault_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
) -> impl Responder {
    if let Some(jwt) = get_user_from_cookie(&req) {
        let con = CONNECTION.lock().unwrap();
        let vault_info = vault_info.into_inner();
        if load_vault(req, web::Json(vault_info.clone()))
            .await
            .is_err()
        {
            return HttpResponse::InternalServerError().body("Failed to get vault");
        }
        let cache = match VAULTS_CACHE.get(&vault_info.get_name()) {
            Some(cache) => cache,
            None => return HttpResponse::InternalServerError().body("Failed to get vault"),
        };
        let vault = cache.lock().unwrap();
        let perms = vault.perms.clone();
        if !perms.contains_key(&jwt.id) || perms.get(&jwt.id).unwrap() < &Perms::Creator {
            return HttpResponse::Unauthorized()
                .body("You do not have permission to delete this vault");
        }

        for id in vault.perms.keys() {
            if remove_vault(&con, &vault_info, *id).is_err() {
                return HttpResponse::InternalServerError().body("Failed to remove vault");
            }
        }

        if fs::remove_dir_all(vault_info.get_path()).is_err() {
            return HttpResponse::InternalServerError().body("Failed to remove vault");
        }

        HttpResponse::Ok().json("")
    } else {
        HttpResponse::Unauthorized().body("Invalid email or password")
    }
}

pub async fn leave_vault_query(
    req: HttpRequest,
    vault_info: web::Json<VaultInfo>,
) -> impl Responder {
    HttpResponse::Ok().json("")
}
