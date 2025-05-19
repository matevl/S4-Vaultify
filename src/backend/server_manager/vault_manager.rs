use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::aes_keys::decrypted_key::decrypt;
use crate::backend::aes_keys::keys_password::{
    derive_key, generate_random_key, generate_salt_from_login,
};
use crate::backend::file_manager::mapping::init_map;
use crate::backend::server_manager::account_manager::{get_user_by_email, Perms, VaultForm};
use crate::backend::server_manager::global_manager::{
    get_user_from_cookie, is_vault_in_cache, CONNECTION, ROOT, SESSION_CACHE, VAULTS_CACHE,
};
use crate::backend::{VAULTS_DATA, VAULT_CONFIG_ROOT, VAULT_USERS_DIR};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const PERMS_PATH: &str = ".vault/perms.json";

type PermsMap = HashMap<u64, Perms>;

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

    pub fn get_name(&self) -> String {
        format!("{}_{}", self.user_id, self.date)
    }

    pub fn get_path(&self) -> String {
        format!(
            "{}/{}{}/.json",
            ROOT.to_str().unwrap(),
            VAULTS_DATA,
            self.get_name(),
        )
    }

    pub async fn create_path(&self) -> Result<(), std::io::Error> {
        let vault_path = self.get_path();
        if let Err(e) = fs::create_dir_all(&vault_path) {
            return Err(e);
        }
        let vault_config = format!("{}{}", vault_path, VAULT_CONFIG_ROOT);
        if let Err(e) = fs::create_dir_all(&vault_config) {
            return Err(e);
        }
        let vault_config_users = format!("{}{}", vault_path, VAULT_USERS_DIR);
        if let Err(e) = fs::create_dir_all(&vault_config_users) {
            return Err(e);
        }

        let vault_perms = format!("{}{}", vault_path, PERMS_PATH);
        if let Err(e) = fs::File::create(&vault_perms) {
            return Err(e);
        }
        Ok(())
    }

    pub async fn save_key(&self, user_key: &[u8], id: u32) -> Result<(), std::io::Error> {
        let key_path = format!("{}{}{}.json", self.get_path(), VAULT_USERS_DIR, id);

        Ok(())
    }

    pub async fn change_key(&self, user_key: &[u8], id: u32) -> Result<(), std::io::Error> {
        Ok(())
    }
    pub async fn set_perms(&self, vault_key: &[u8], perms: &PermsMap) -> Result<(), std::io::Error> {

        if fs::exists()

        Ok(())
    }

    pub async fn get_perms(&self, vault_key: &[u8]) -> Result<PermsMap, &str> {
        let path = format!("{}{}", self.get_path(), PERMS_PATH);
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

pub struct VaultsCache {
    info: VaultInfo,
    perms: PermsMap,
    vault_key: Vec<u8>,
}

impl VaultsCache {
    pub fn new(info: &VaultInfo, perms: &PermsMap, vault_key: &Vec<u8>) -> Self {
        VaultsCache {
            info: info.clone(),
            perms: perms.clone(),
            vault_key: vault_key.clone(),
        }
    }
}

/**
 * Creates a new vault for a user.
 *
 * @param conn - The database connection.
 * @param vault_info - The information about the vault.
 * @return A Result containing the ID of the created vault.
 */

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
        if let Some(session) = SESSION_CACHE.get(&decoded_jwt.session_id) {
            let session = session.lock().unwrap();

            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let info = VaultInfo::new(decoded_jwt.id, &form.name, time);

            match info.create_path().await {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to create user JSON file: {:?}", e);
                    return HttpResponse::InternalServerError()
                        .body("Failed to create user JSON file.");
                }
            }

            let vault_key = generate_random_key();

            let vault_key = derive_key(
                &String::from_utf8_lossy(vault_key.as_slice()),
                generate_salt_from_login(&decoded_jwt.email).as_slice(),
                10000,
            );

            let content = serde_json::to_string(&(vault_key, Perms::Creator)).unwrap();
            let encrypted_content = encrypt(content.as_bytes(), session.user_key.as_slice());
            fs::write(&user_json, &encrypted_content).unwrap();

            init_map(
                &format!("{}/map.json", vault_path),
                session.user_key.as_slice(),
            );

            match create_vault(&connection, &info) {
                Ok(res) => HttpResponse::Ok().json(json!({
                    "message": format!("Coffre '{}' créé avec succès !", res.name),
                    "vault_id": res.user_id.clone(),
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
        if is_vault_in_cache(&info.get_name()).await {
            HttpResponse::Ok().json(info.clone())
        } else if let Some(session) = SESSION_CACHE.get(&jwt.session_id) {
            let mut session = session.lock().unwrap();

            let vault_name = info.get_name();
            let key_path = format!("{}{}{}.json", info.get_path(), VAULT_USERS_DIR, jwt.id);

            let encrypted_content = match fs::read(&key_path) {
                Ok(data) => data,
                Err(_) => return HttpResponse::NotFound().body("Vault file not found"),
            };

            let decrypted_content =
                match decrypt(encrypted_content.as_slice(), session.user_key.as_slice()) {
                    Ok(data) => data,
                    Err(_) => return HttpResponse::InternalServerError().body("Failed to decrypt"),
                };

            let vault_key: Vec<u8> =
                match serde_json::from_str(&String::from_utf8(decrypted_content).unwrap()) {
                    Ok(parsed) => parsed,
                    Err(_) => {
                        return HttpResponse::InternalServerError().body("Invalid vault data")
                    }
                };

            let vault_perms: PermsMap = match info.get_perms(&vault_key).await {
                Ok(perms) => perms,
                Err(_) => {
                    return HttpResponse::InternalServerError().body("Invalid vault permissions")
                }
            };

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

pub async fn get_perms_query(req: HttpRequest, vault_info: web::Json<VaultInfo>) -> impl Responder {
    HttpResponse::Ok().json("")
}

pub async fn share_vault_query(
    req: HttpRequest,
    data: web::Json<(VaultInfo, String)>,
) -> impl Responder {
    HttpResponse::Ok().json("")
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
