mod models;


use actix_web::{web, App, HttpServer,HttpResponse, Responder};
use actix_web::middleware::Logger;
use models::*;
use serde::{Deserialize, Serialize};
use s4_vaultify::backend::*;
use s4_vaultify::backend::account_manager::account::{load_users_data, local_log_in, UserInput, JWT};
use s4_vaultify::backend::vault_manager::init_config_vaultify;
// Structure pour les identifiants utilisateur
#[derive(Deserialize)]
struct LoginData {
    email: String,
    password: String,
}
impl LoginData {
    fn new(email: String, password: String) -> LoginData {
        LoginData { email, password }
    }
    fn get_email(&self) -> &String {
        &self.email
    }
    fn get_password(&self) -> &String {
        &self.password
    }
}
#[derive(Deserialize)]
struct RegisterData {
    email: String,
    password: String,
}
// Structure pour la réponse JWT
async fn login(user_data: web::Json<UserInput>) -> impl Responder {
    // Charger les utilisateurs depuis le fichier
    let users_data = load_users_data("path_to_users_data");

    // Vérifier les identifiants de l'utilisateur avec UserInput
    match local_log_in(&user_data.into_inner(), users_data) {
        Ok(jwt) => HttpResponse::Ok().json(jwt), // Retourne le JWT
        Err(_) => HttpResponse::Unauthorized().body("Invalid credentials"),
    }
}


// Handler pour une page protégée
async fn protected_route() -> impl Responder {
    HttpResponse::Ok().body("Bienvenue dans la page protégée !")
}

async fn register(register_data: web::Json<RegisterData>) -> impl Responder {
    // Charger les utilisateurs depuis le fichier
    let mut users_data = load_users_data("path_to_users_data");

    // Ajouter l'utilisateur aux données
    let user_input = UserInput::new(register_data.email.clone(), register_data.password.clone());
    match add_user_to_data(user_input, &mut users_data, Perms::Read) {
        Ok(_) => {
            sava_users_data(&users_data, "path_to_users_data"); // Sauvegarder les données après ajout
            HttpResponse::Created().body("User created successfully")
        }
        Err(_) => HttpResponse::Conflict().body("User already exists"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/login", web::post().to(login))
            .route("/register", web::post().to(register))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}