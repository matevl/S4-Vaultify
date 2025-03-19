use crate::backend::aes_keys::keys_password::{
    derive_key, generate_random_key, generate_salt_from_login,
};
use crate::backend::{VAULT_CONFIG_ROOT, VAULT_USERS_DIR};
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use dirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    pub static ref USERS_DB: Arc<Mutex<UsersData>> = Arc::new(Mutex::new(UsersData::new()));
    pub static ref VAULT_ACESS: Arc<Mutex<VaultsAccess>> =
        Arc::new(Mutex::new(VaultsAccess::new()));
    pub static ref PRIVATE_DATA: Arc<Mutex<PrivateData>> = Arc::new(Mutex::new(PrivateData::new()));
    pub static ref ROOT: std::path::PathBuf = dirs::home_dir().expect("Could not find home dir");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Perms {
    Admin,
    Write,
    Read,
}

/**
 * Name -> (HashPw, id)
 */
type UsersData = HashMap<String, (String, u32)>;

/**
 * Name -> (Vault_Name -> Realpath (vault_id in string))
 */
type VaultsAccess = HashMap<String, Vec<VaultInfo>>;

/**
 *
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VaultInfo {
    pub name: String,
    pub path: String,
    pub date: u64,
}

impl VaultInfo {
    pub fn new(name: &String, path: &String, date: u64) -> Self {
        Self {
            name: name.clone(),
            path: path.clone(),
            date,
        }
    }
}

/**
 * ID -> JWTPrivate
 */
type PrivateData = HashMap<u32, JWTPrivate>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserJson {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JWT {
    id: u32,
    email: String,
    loaded_vault: Option<VaultInfo>,
}

impl JWT {
    pub fn new(id: u32, email: &String) -> JWT {
        JWT {
            id,
            email: email.clone(),
            loaded_vault: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JWTPrivate {
    hash_pw: String,
    user_key: Box<[u8]>,
    vault_key: Box<[u8]>,
}

impl JWTPrivate {
    pub fn new(hash_pw: &String, user_key: &[u8]) -> JWTPrivate {
        JWTPrivate {
            hash_pw: hash_pw.clone(),
            user_key: user_key.to_vec().into_boxed_slice(),
            vault_key: Box::new([]),
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
        .ok_or_else(|| HttpResponse::InternalServerError());

    fs::create_dir_all(ROOT.join(new_id.to_string())).unwrap();

    let salt = generate_salt_from_login(user.email.as_str());
    let key = derive_key(user.password.as_str(), salt.as_slice(), 10000);

    let mut private_data = PRIVATE_DATA.lock().unwrap();
    private_data.insert(new_id, JWTPrivate::new(&hash_pw, &key));

    HttpResponse::Ok().json(JWT::new(new_id, &email))
}

#[actix_web::post("/user/login")]
async fn login_user_query(user: web::Json<UserJson>) -> impl Responder {
    let email = user.email.clone();
    let pw = user.password.clone();
    let mut db = USERS_DB.lock().unwrap();

    let data = db.get(&email).unwrap_or(&("".to_string(), 0)).clone();

    if data.0.len() > 0 && verify(&pw, &data.0).is_ok() {
        let salt = generate_salt_from_login(user.email.as_str());
        let key = derive_key(user.password.as_str(), salt.as_slice(), 10000);

        let mut private_data = PRIVATE_DATA.lock().unwrap();
        private_data.insert(data.1, JWTPrivate::new(&data.0, &key));
        HttpResponse::Ok().json(JWT::new(data.1, &email))
    } else {
        HttpResponse::NotFound().finish()
    }
}

async fn get_vaults_list_query(user: web::Json<JWT>) -> impl Responder {
    let access = VAULT_ACESS.lock().unwrap();
    let vaults = access.get(&user.email).unwrap();

    HttpResponse::Ok().json(vaults.clone())
}

async fn create_vault_query(mut jwt: web::Json<JWT>, name: web::Json<String>) -> impl Responder {
    let mut vault_access = VAULT_ACESS.lock().unwrap();

    let user_acces = vault_access.get_mut(jwt.email.as_str()).unwrap();

    let private_jwt = PRIVATE_DATA.lock().unwrap().get_mut(&jwt.id).unwrap();

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let vault_path = format!("{}{}{}", ROOT.to_str().unwrap(), jwt.id, time);
    let vault_config_path = format!("{}{}", vault_path, VAULT_CONFIG_ROOT);
    let vault_key_data = format!("{}{}", vault_path, VAULT_USERS_DIR);
    let key_path = format!("{}{}.json", vault_key_data, jwt.id);

    fs::create_dir_all(&vault_path).unwrap();
    fs::create_dir_all(&vault_config_path).unwrap();
    fs::create_dir_all(&vault_key_data).unwrap();
    let mut file = fs::File::create(&key_path).unwrap();

    let salt = generate_salt_from_login(jwt.email.as_str());
    let vault_key = derive_key(
        &String::from_utf8(generate_random_key()).unwrap(),
        &salt,
        10000,
    );

    file.write_all(serde_json::to_string(&vault_key).unwrap().as_bytes())
        .unwrap();

    let mut info = VaultInfo::new(&name, &vault_path, time);
    user_acces.push(info.clone());

    private_jwt.vault_key = vault_key;

    jwt.loaded_vault = Some(info.clone());
    HttpResponse::Ok().json(jwt)
}

async fn load_vault_query(mut jwt: web::Json<JWT>, info: web::Json<VaultInfo>) -> impl Responder {
    let access = VAULT_ACESS.lock().unwrap().get(jwt.email.as_str()).unwrap();
    let private_jwt = PRIVATE_DATA.lock().unwrap().get_mut(&jwt.id).unwrap();

    let key_path = format!(
        "{}{}{}{}.json",
        ROOT.to_str().unwrap(),
        info.path,
        VAULT_USERS_DIR,
        jwt.id
    );

    let mut content = fs::read_to_string(&key_path).unwrap();
    let vault_key: Box<[u8]> = serde_json::from_str(&content).unwrap();
    private_jwt.vault_key = vault_key;

    jwt.loaded_vault = Some(info.clone());
    HttpResponse::Ok().json(jwt)
}
