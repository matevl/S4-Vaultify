use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use bcrypt::{hash, DEFAULT_COST};
use lazy_static::lazy_static;
use s4_vaultify::backend::account_manager::account::{
    add_user_to_data, load_users_data, load_vault_matching, log_to_vaultify, UserData, UserInput,
    JWT,
};
use s4_vaultify::backend::account_manager::account_server::{
    JWTPrivate, UserJson, PRIVATE_DATA, ROOT, USERS_DB,
};
use s4_vaultify::backend::aes_keys::keys_password::{derive_key, generate_salt_from_login};
use s4_vaultify::backend::VAULTIFY_CONFIG;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref USERD: Mutex<Vec<UserData>> = Mutex::new(load_users_data());
    static ref VAULT_MATCh: Mutex<HashMap<String, Vec<(String, String)>>> =
        Mutex::new(load_vault_matching());
}

#[derive(Serialize, Deserialize)]
struct LogInput {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct CreateUserResponse {
    id: u32,
    name: String,
    wuser_data: WuserData,
}

#[derive(Serialize, Deserialize)]
struct WuserData {
    id: u32,
    user_input: UserInput,
    jwt: JWT,
}

impl WuserData {
    pub(crate) fn clone(&self) -> WuserData {
        WuserData {
            id: self.id,
            user_input: self.user_input.clone(),
            jwt: self.jwt.clone(),
        }
    }
}

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

    HttpResponse::Ok()
        .json(s4_vaultify::backend::account_manager::account_server::JWT::new(new_id, &email))
}

#[actix_web::post("/user/log")]
async fn log_user(user_data: web::Json<LogInput>, db: web::Data<UserDb>) -> impl Responder {
    let mut db = db.lock().unwrap();
    let new_id = db.keys().max().unwrap_or(&0) + 1;
    let mail = user_data.email.clone();
    let password = user_data.password.clone();
    let usrinput = UserInput::new(mail.clone(), password.clone());
    let jwt = log_to_vaultify(&usrinput, &mut *USERD.lock().unwrap())
        .expect("Error while adding user to vaultify");
    db.insert(
        new_id,
        WuserData {
            id: new_id,
            user_input: usrinput.clone(),
            jwt: jwt.clone(),
        },
    );
    HttpResponse::Created().json(CreateUserResponse {
        id: new_id,
        name: mail,
        wuser_data: WuserData {
            id: new_id,
            user_input: usrinput.clone(),
            jwt: jwt.clone(),
        },
    })
}

type UserDb = Arc<Mutex<HashMap<u32, WuserData>>>;
#[actix_web::get("/greet/{name}")]
async fn greet(user_id: web::Path<u32>) -> impl Responder {
    format!("Hello {user_id}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    let user_db: UserDb = Arc::new(Mutex::new(HashMap::<u32, WuserData>::new()));
    println!("Starting server on port {}", port);
    HttpServer::new(move || {
        let app_data = web::Data::new(user_db.clone());
        App::new()
            .app_data(app_data)
            .service(create_user_query)
            .service(greet)
            .service(log_user)
    })
    .bind(format!("127.0.0.1:{}", port))?
    .workers(2)
    .run()
    .await
}
