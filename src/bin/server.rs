use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Error};
use actix_web::middleware::Logger;
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use futures::future::{ok, Ready};
use futures::FutureExt;
use jsonwebtoken::{decode, Validation, DecodingKey};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use spin::Mutex;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use s4_vaultify::backend::account_manager::account::{
    add_user_to_data, load_users_data, load_vault_matching, log_to_vaultify, save_users_data,
    UserData, UserInput, JWT,
};
use s4_vaultify::backend::vault_manager::init_config_vaultify;

lazy_static! {
    static ref USERD: Mutex<Vec<UserData>> = {
        init_config_vaultify();
        Mutex::new(load_users_data())
    };
    static ref VAULT_MATCH: Mutex<HashMap<String, Vec<(String, String)>>> = Mutex::new(load_vault_matching());
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Le sujet du JWT (par exemple, l'ID utilisateur ou l'email)
    pub exp: usize,   // Expiration du JWT
}

#[derive(Deserialize)]
struct Choice {
    choice: String,
}

async fn homepage(choice: web::Query<Choice>) -> impl Responder {
    match choice.choice.as_str() {
        "1" => HttpResponse::Found().header("Location", "/auth/register").finish(),
        "2" => HttpResponse::Found().header("Location", "/auth/login").finish(),
        _ => HttpResponse::BadRequest().body("Choix invalide, veuillez réessayer."),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap_fn(auth_middleware)
            .service(Files::new("/", "./static").index_file("index.html"))
            .route("/", web::get().to(homepage))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(register_user))
                    .route("/login", web::post().to(login_user))
                    .route("/myvaults", web::get().to(myvaults)),
            )
            .route("/account", web::get().to(account_details))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

// Fonction pour enregistrer un utilisateur
async fn register_user(info: web::Json<RegisterRequest>) -> impl Responder {
    let email = &info.email;
    if !is_valid_email(email) {
        return HttpResponse::BadRequest().body("Email invalide, veuillez réessayer.");
    }

    let password = &info.password;

    let user_input = UserInput {
        email: email.trim().to_string(),
        password: password.trim().to_string(),
    };

    add_user_to_data(&user_input, USERD.lock().deref_mut());

    let jwt = log_to_vaultify(&user_input, USERD.lock().deref());
    HttpResponse::Found()
        .header("Location", "/auth/myvaults")
        .finish()
}

// Fonction pour connecter un utilisateur
async fn login_user(info: web::Json<LoginRequest>) -> impl Responder {
    let email = &info.email;
    if !is_valid_email(email) {
        return HttpResponse::BadRequest().body("Email invalide, veuillez réessayer.");
    }

    let password = &info.password;

    let user_input = UserInput {
        email: email.trim().to_string(),
        password: password.trim().to_string(),
    };

    let jwt = log_to_vaultify(&user_input, USERD.lock().deref()).expect("Erreur lors de la connexion");

    HttpResponse::Found()
        .header("Location", "/auth/myvaults")
        .json(jwt)
}

// Fonction pour vérifier si l'email est valide
fn is_valid_email(email: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}

async fn myvaults() -> impl Responder {
    HttpResponse::Ok().body("Vos vaults sont ici")
}

async fn account_details() -> impl Responder {
    HttpResponse::Ok().body("Détails du compte utilisateur")
}

// Middleware d'authentification
async fn auth_middleware(
    req: ServiceRequest,
    srv: &ServiceRequest,
) -> Result<ServiceRequest, Error> {
    let auth_header = req.headers().get("Authorization").cloned();
    if let Some(header) = auth_header {
        if let Ok(token) = header.to_str() {
            let token = token.trim_start_matches("Bearer ").to_string();
            let decoding_key = Arc::new(DecodingKey::from_secret("secret_key".as_ref()));
            let validation = Arc::new(Validation::default());

            let decoded = web::block(move || decode_token(token, &decoding_key, &validation)).await;
            match decoded {
                Ok(_) => Ok(req),
                Err(_) => Err(ErrorUnauthorized("Token invalide")),
            }
        } else {
            Err(ErrorUnauthorized("Token invalide"))
        }
    } else {
        Err(ErrorUnauthorized("Pas d'authentification"))
    }
}

// Fonction auxiliaire pour effectuer le décodage
fn decode_token(token: String, decoding_key: &DecodingKey, validation: &Validation) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(&token, decoding_key, validation).map(|data| data.claims)
}
