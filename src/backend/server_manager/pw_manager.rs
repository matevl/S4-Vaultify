use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::aes_keys::decrypted_key::decrypt;
use crate::backend::VAULTS_DATA;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::backend::server_manager::account_manager::{JWT, ROOT, SESSION_CACHE};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PasswordEntry {
    pub email: String,
    pub identifiant: String,
    pub platform: String,
    pub password: String,
}

fn get_passwords_path(user_id: u32) -> PathBuf {
    let mut path = ROOT.clone();
    path.push(format!("{}{}password.json", VAULTS_DATA, user_id));
    path
}

pub async fn get_user_passwords(user: web::Json<JWT>) -> impl Responder {
    let path = get_passwords_path(user.id);

    if let Some(session) = SESSION_CACHE.get(&user.session_id) {
        if let Ok(encrypted_data) = fs::read(&path) {
            if let Ok(decrypted_data) = decrypt(&encrypted_data, session.user_key.as_slice()) {
                if let Ok(passwords) = serde_json::from_slice::<Vec<PasswordEntry>>(&decrypted_data)
                {
                    return HttpResponse::Ok().json(passwords);
                }
            }
        }
        HttpResponse::Ok().json(Vec::<PasswordEntry>::new())
    } else {
        HttpResponse::Unauthorized().body("Invalid session")
    }
}

pub async fn add_user_password(data: web::Json<(JWT, PasswordEntry)>) -> impl Responder {
    let (jwt, new_entry) = data.into_inner();
    let path = get_passwords_path(jwt.id);

    if let Some(session) = SESSION_CACHE.get(&jwt.session_id) {
        let mut passwords: Vec<PasswordEntry> = if let Ok(encrypted_data) = fs::read(&path) {
            if let Ok(decrypted_data) = decrypt(&encrypted_data, session.user_key.as_slice()) {
                serde_json::from_slice(&decrypted_data).unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        passwords.push(new_entry);

        let json_data = serde_json::to_vec(&passwords).unwrap();
        let encrypted_data = encrypt(&json_data, session.user_key.as_slice());
        fs::write(&path, &encrypted_data).unwrap();
        HttpResponse::Ok().body("Password added")
    } else {
        HttpResponse::Unauthorized().body("Invalid session")
    }
}

pub async fn remove_user_password(data: web::Json<(JWT, PasswordEntry)>) -> impl Responder {
    let (jwt, password_to_remove) = data.into_inner();
    let path = get_passwords_path(jwt.id);

    if let Some(session) = SESSION_CACHE.get(&jwt.session_id) {
        let mut passwords: Vec<PasswordEntry> = if let Ok(encrypted_data) = fs::read(&path) {
            if let Ok(decrypted_data) = decrypt(&encrypted_data, session.user_key.as_slice()) {
                serde_json::from_slice(&decrypted_data).unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        passwords.retain(|entry| entry != &password_to_remove);

        let json_data = serde_json::to_vec(&passwords).unwrap();
        let encrypted_data = encrypt(&json_data, session.user_key.as_slice());
        fs::write(&path, &encrypted_data).unwrap();
        HttpResponse::Ok().body("Password removed")
    } else {
        HttpResponse::Unauthorized().body("Invalid session")
    }
}
