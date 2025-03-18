use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref USERS_DB: Arc<Mutex<UsersData>> = Arc::new(Mutex::new(UsersData::new()));
    pub static ref VAULT_ACESS: Arc<Mutex<VaultsAccess>> =
        Arc::new(Mutex::new(VaultsAccess::new()));
}

/**
 * Name -> (HashPw, id)
 */

type UsersData = HashMap<String, (String, u32)>;

/**
 * Name -> (Vault_Name -> Realpath (vault_id in string))
 */
type VaultsAccess = HashMap<String, HashMap<String, String>>;

pub struct UserJson {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JWT {
    id: u32,
    email: String,
    hash_pw: String,
}

impl JWT {
    pub fn new(id: u32, email: String, hash_pw: String) -> JWT {
        JWT {
            id,
            email: email.clone(),
            hash_pw: hash_pw.clone(),
        }
    }
}

#[actix_web::post("/user/register")]
async fn create_user_query(user: web::Json<UserJson>) -> impl Responder {
    let email = user.email.clone();
    let pw = user.password.clone();

    let mut db = USERS_DB.lock().unwrap();
    let new_id = db.values().last().unwrap_or(&("".to_string(), 0)).1 + 1;

    let hash_pw = hash(pw.clone(), DEFAULT_COST).unwrap();

    db.insert(email.clone(), (hash_pw.clone(), new_id))
        .ok_or_else(|| HttpResponse::InternalServerError())?;

    HttpResponse::Ok().json(JWT::new(new_id, email, hash_pw));
}

#[actix_web::post("/user/login")]
async fn login_user_query(user: web::Json<UserJson>) -> impl Responder {
    let email = user.email.clone();
    let pw = user.password.clone();
    let mut db = USERS_DB.lock().unwrap();

    let data = db.get(&email).unwrap_or(&("".to_string(), 0)).clone();

    if data.0.len() > 0 && verify(&pw, &data.0).is_ok() {
        HttpResponse::Ok().json(JWT::new(data.1, email, data.0))
    } else {
        HttpResponse::NotFound().finish()
    }
}

async fn get_vaults_list_query(user: web::Json<JWT>) -> impl Responder {
    let access = VAULT_ACESS.lock().unwrap();
    let vaults = access
        .get(&user.email)
        .ok_or_else(|| HttpResponse::NotFound())?;

    HttpResponse::Ok().json(vaults.clone())
}
