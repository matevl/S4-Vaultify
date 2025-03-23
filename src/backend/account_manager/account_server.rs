use crate::backend::aes_keys::keys_password::{
    derive_key, generate_random_key, generate_salt_from_login,
};
use crate::backend::{
    USERS_DATA, VAULTIFY_CONFIG, VAULTS_MATCHING, VAULT_CONFIG_ROOT, VAULT_USERS_DIR,
};
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use dirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/**
 * Lazy static initialization for global variables.
 * These variables are used to store user data, vault access, and private data.
 */
lazy_static! {
    /**
     * Global user database.
     */
    pub static ref USERS_DB: Arc<Mutex<UsersData>> = {
        init_server_config();
        Arc::new(Mutex::new(load_user_data()))
    };

    /**
     * Global vault access data.
     */
    pub static ref VAULT_ACESS: Arc<Mutex<VaultsAccess>> =
        Arc::new(Mutex::new(load_vault_matching()));

    /**
     * Global private data storage.
     */
    pub static ref PRIVATE_DATA: Arc<Mutex<PrivateData>> = Arc::new(Mutex::new(PrivateData::new()));

    /**
     * Root directory path for the application.
     */
    pub static ref ROOT: std::path::PathBuf = dirs::home_dir().expect("Could not find home dir");
}

/**
 * Enum to handle permission verification.
 * Each variant represents a different level of access.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Perms {
    Admin,
    Write,
    Read,
}

#[allow(dead_code)]
impl Perms {
    /**
     * Check if the permission allows reading.
     */
    pub fn can_read(&self) -> bool {
        matches!(self, Perms::Admin | Perms::Write | Perms::Read)
    }

    /**
     * Check if the permission allows writing.
     */
    pub fn can_write(&self) -> bool {
        matches!(self, Perms::Admin | Perms::Write)
    }

    /**
     * Check if the permission allows execution (admin-only).
     */
    pub fn can_execute(&self) -> bool {
        matches!(self, Perms::Admin)
    }
}

/**
 * Type alias for user data storage.
 * Maps user names to their hashed passwords and IDs.
 */
pub type UsersData = HashMap<String, (String, u32)>;

/**
 * Type alias for vault access storage.
 * Maps user names to their accessible vaults.
 */
pub type VaultsAccess = HashMap<String, Vec<VaultInfo>>;

/**
 * Struct representing vault information.
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VaultInfo {
    pub name: String,
    pub path: String,
    pub date: u64,
}

impl VaultInfo {
    /**
     * Create a new VaultInfo instance.
     */
    pub fn new(name: &String, path: &String, date: u64) -> Self {
        Self {
            name: name.clone(),
            path: path.clone(),
            date,
        }
    }
}

/**
 * Type alias for private data storage.
 * Maps user IDs to their private JWT data.
 */
type PrivateData = HashMap<u32, JWTPrivate>;

/**
 * Struct representing user JSON data for API requests.
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserJson {
    pub email: String,
    pub password: String,
}

/**
 * Struct representing a JSON Web Token (JWT).
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JWT {
    pub id: u32,
    pub email: String,
    pub loaded_vault: Option<VaultInfo>,
}

impl JWT {
    /**
     * Create a new JWT instance.
     */
    pub fn new(id: u32, email: &String) -> JWT {
        JWT {
            id,
            email: email.clone(),
            loaded_vault: None,
        }
    }
}

/**
 * Struct representing private JWT data.
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JWTPrivate {
    hash_pw: String,
    user_key: Box<[u8]>,
    vault_key: Box<[u8]>,
    vault_perms: Perms,
}

impl JWTPrivate {
    /**
     * Create a new JWTPrivate instance.
     */
    pub fn new(hash_pw: &String, user_key: &[u8]) -> JWTPrivate {
        JWTPrivate {
            hash_pw: hash_pw.clone(),
            user_key: user_key.to_vec().into_boxed_slice(),
            vault_key: Box::new([]),
            vault_perms: Perms::Read,
        }
    }
}

/**
 * Endpoint to create a new user.
 */
#[actix_web::post("/user/register")]
pub async fn create_user_query(user: web::Json<UserJson>) -> impl Responder {
    let email = user.email.clone();
    let pw = user.password.clone();

    let mut db = USERS_DB.lock().unwrap();
    let new_id = db.values().last().unwrap_or(&("".to_string(), 0)).1 + 1;

    let hash_pw = hash(pw.clone(), DEFAULT_COST).unwrap();

    db.insert(email.clone(), (hash_pw.clone(), new_id)).unwrap();

    fs::create_dir_all(ROOT.join(new_id.to_string())).unwrap();

    let salt = generate_salt_from_login(user.email.as_str());
    let key = derive_key(user.password.as_str(), salt.as_slice(), 10000);

    let mut private_data = PRIVATE_DATA.lock().unwrap();
    private_data.insert(new_id, JWTPrivate::new(&hash_pw, &key));

    HttpResponse::Ok().json(JWT::new(new_id, &email))
}

/**
 * Endpoint to log in a user.
 */
#[actix_web::post("/user/login")]
pub async fn login_user_query(user: web::Json<UserJson>) -> impl Responder {
    let email = user.email.clone();
    let pw = user.password.clone();
    let db = USERS_DB.lock().unwrap();

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

/**
 * Endpoint to get the list of vaults for a user.
 */
pub async fn get_vaults_list_query(user: web::Json<JWT>) -> impl Responder {
    let access = VAULT_ACESS.lock().unwrap();
    let vaults = access.get(&user.email).unwrap();

    HttpResponse::Ok().json(vaults.clone())
}

/**
 * Endpoint to create a new vault for a user.
 */
pub async fn create_vault_query(
    mut jwt: web::Json<JWT>,
    name: web::Json<String>,
) -> impl Responder {
    let mut vault_access = VAULT_ACESS.lock().unwrap();

    let user_acces = vault_access.get_mut(jwt.email.as_str()).unwrap();

    let mut private_data = PRIVATE_DATA.lock().unwrap();
    let private_jwt = private_data.get_mut(&jwt.id).unwrap();

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

    file.write(
        serde_json::to_string(&(&vault_key, Perms::Admin))
            .unwrap()
            .as_bytes(),
    )
    .unwrap();

    let info = VaultInfo::new(&name, &vault_path, time);
    user_acces.push(info.clone());

    private_jwt.vault_key = vault_key.into_boxed_slice();

    jwt.loaded_vault = Some(info.clone());
    HttpResponse::Ok().json(jwt)
}

/**
 * Endpoint to load a vault for a user.
 */
pub async fn load_vault_query(
    mut jwt: web::Json<JWT>,
    info: web::Json<VaultInfo>,
) -> impl Responder {
    let mut private_data = PRIVATE_DATA.lock().unwrap();
    let private_jwt = private_data.get_mut(&jwt.id).unwrap();

    let key_path = format!(
        "{}{}{}{}.json",
        ROOT.to_str().unwrap(),
        info.path,
        VAULT_USERS_DIR,
        jwt.id
    );

    let content = fs::read_to_string(&key_path).unwrap();
    let (vault_key, vault_perms): (Box<[u8]>, Perms) = serde_json::from_str(&content).unwrap();
    private_jwt.vault_key = vault_key;
    private_jwt.vault_perms = vault_perms;

    jwt.loaded_vault = Some(info.clone());
    HttpResponse::Ok().json(jwt)
}

/**
 * Initialize the server configuration.
 */
pub fn init_server_config() {
    let config_root = format!("{}{}", ROOT.to_str().unwrap(), VAULTIFY_CONFIG);
    if !fs::exists(&config_root).is_ok() {
        fs::create_dir_all(&config_root).expect("Could not create folder");
    }
    let user_data = format!("{}{}", config_root, USERS_DATA);
    if !fs::exists(&user_data).is_ok() {
        let mut file = fs::File::create(&user_data).expect("Could not create file");
        file.write_all(serde_json::to_string(&UsersData::new()).unwrap().as_bytes())
            .unwrap();
    }
    let vault_matching = format!("{}{}", config_root, VAULTS_MATCHING);
    if !fs::exists(&vault_matching).is_ok() {
        let mut file = fs::File::create(&vault_matching).expect("Could not create file");
        file.write_all(
            serde_json::to_string(&VaultsAccess::new())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
    }
}

/**
 * Load user data from the file system.
 */
pub fn load_user_data() -> UsersData {
    let user_data = format!("{}{}", ROOT.to_str().unwrap(), USERS_DATA);
    let content = fs::read_to_string(user_data).unwrap();
    serde_json::from_str::<UsersData>(&content).unwrap()
}

/**
 * Load vault matching data from the file system.
 */
pub fn load_vault_matching() -> VaultsAccess {
    let user_data = format!("{}{}", ROOT.to_str().unwrap(), VAULTS_MATCHING);

    // Vérifiez si le fichier existe avant d'essayer de le lire
    if fs::metadata(&user_data).is_ok() {
        match fs::read_to_string(&user_data) {
            Ok(content) => {
                // Affichez le contenu pour vérifier son format
                println!("Contenu du fichier : {}", content);

                match serde_json::from_str::<VaultsAccess>(&content) {
                    Ok(vault_access) => vault_access,
                    Err(e) => panic!("Failed to parse JSON: {}", e),
                }
            },
            Err(e) => panic!("Failed to read file: {}", e),
        }
    } else {
        // Si le fichier n'existe pas, retournez une structure par défaut ou gérez l'erreur
        VaultsAccess::new()
    }
}



/**
 * Endpoint to save the server configuration.
 */
pub async fn save_server_config() -> impl Responder {
    let users_db = USERS_DB.lock().unwrap();
    let vault_access = VAULT_ACESS.lock().unwrap();

    let path_users_db = format!("{}{}", ROOT.to_str().unwrap(), USERS_DATA);
    let path_vault_access = format!("{}{}", ROOT.to_str().unwrap(), VAULTS_MATCHING);
    fs::write(
        &path_users_db,
        serde_json::to_string(&users_db.deref()).unwrap().as_bytes(),
    )
    .unwrap();
    fs::write(
        &path_vault_access,
        &serde_json::to_string(&vault_access.deref())
            .unwrap()
            .as_bytes(),
    )
    .unwrap();

    HttpResponse::Ok().json("Saving server config successfully.")
}
