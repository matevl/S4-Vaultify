use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::aes_keys::decrypted_key::decrypt;
use crate::backend::aes_keys::keys_password::{
    derive_key, generate_random_key, generate_salt_from_login,
};
use crate::backend::file_manager::mapping::init_map;
use crate::backend::server_manager::account_manager::{Perms, VaultForm, VaultInfo};
use crate::backend::server_manager::global_manager::{
    get_user_from_cookie, CONNECTION, ROOT, SESSION_CACHE,
};
use crate::backend::{VAULTS_DATA, VAULT_CONFIG_ROOT, VAULT_USERS_DIR};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use rusqlite::{params, Connection};
use serde_json::json;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/**
 * Creates a new vault for a user.
 *
 * @param conn - The database connection.
 * @param vault_info - The information about the vault.
 * @return A Result containing the ID of the created vault.
 */

pub fn create_vault(conn: &Connection, vault_info: &VaultInfo) -> rusqlite::Result<VaultInfo> {
    if let Ok(val) = conn.execute(
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
        let name: &str = &form.name;
        if let Some(cache) = SESSION_CACHE.get(&decoded_jwt.session_id) {
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
            match fs::File::create(&user_json) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Failed to create user JSON file: {:?}", e);
                    return HttpResponse::InternalServerError()
                        .body("Failed to create user JSON file.");
                }
            }
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
        if let Some(mut cache) = SESSION_CACHE.get(&jwt.session_id) {
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
