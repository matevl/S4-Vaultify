use crate::backend::account_manager::account_server::{UserJson, JWT, PRIVATE_DATA, ROOT};
use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::{PASSWORD, VAULTIFY_DATABASE, VAULTS_DATA};
use actix_web;
use actix_web::{web, HttpResponse, Responder};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sha2::digest::typenum::private::IsNotEqualPrivate;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref PASSWORD_MANAGER: Arc<Mutex<PasswordManager>> = {
        let manager = PasswordManager::new();
        Arc::new(Mutex::new(manager))
    };
}

type PasswordManager = HashMap<u32, Vec<Password>>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Password {
    pub username: String,
    pub email: String,
    pub password: String,
    pub web_site: String,
}

pub async fn get_passwords(jwt: web::Json<JWT>) -> impl Responder {
    let password_manager = PASSWORD_MANAGER.lock().unwrap();
    let data = password_manager.get(&jwt.id).unwrap();
    HttpResponse::Ok().json(data.clone())
}
pub async fn add_passwords(jwt: web::Json<JWT>, password: web::Json<Password>) -> impl Responder {
    let mut password_manager = PASSWORD_MANAGER.lock().unwrap();
    let data = password_manager.get_mut(&jwt.id).unwrap();

    data.push(password.clone());
    HttpResponse::Ok().json(data.clone())
}

pub async fn remove_passwords(
    jwt: web::Json<JWT>,
    password: web::Json<Password>,
) -> impl Responder {
    let mut password_manager = PASSWORD_MANAGER.lock().unwrap();
    let data = password_manager.get_mut(&jwt.id).unwrap();

    let clone = data.clone();
    data.clear();
    for c in clone {
        if PartialEq::ne(&c, &password) {
            data.push(c.clone());
        }
    }
    HttpResponse::Ok().json(data.clone())
}

pub async fn save_password(jwt: web::Json<JWT>) -> impl Responder {
    let password_manager = PASSWORD_MANAGER.lock().unwrap();
    let data = password_manager.get(&jwt.id).unwrap();

    let private_datas = PRIVATE_DATA.lock().unwrap();
    let private_data = private_datas.get(&jwt.id).unwrap();

    let path = format!(
        "{}{}{}/{}",
        ROOT.to_str().unwrap(),
        VAULTS_DATA,
        jwt.id,
        PASSWORD
    );

    let contend = serde_json::to_string(&data).unwrap();
    let encrypted_contend = encrypt(contend.as_bytes(), &private_data.user_key);

    fs::write(path, &encrypted_contend).unwrap();

    HttpResponse::Ok().json(true)
}
