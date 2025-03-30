use actix_files::NamedFile;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use s4_vaultify::backend::account_manager::account_server::*;
use s4_vaultify::backend::aes_keys::keys_password::{derive_key, generate_salt_from_login};
use s4_vaultify::backend::VAULTS_DATA;
use std::fs;

// Gestion des formulaires (POST)

// Définition des structures des données du formulaire
#[derive(serde::Deserialize, Debug)]
struct CreateUserForm {
    username: String,
    password: String,
}

#[derive(serde::Deserialize, Debug)]
struct LoginForm {
    username: String,
    password: String,
}

// Fonction pour afficher la page HTML de création d'utilisateur
async fn create_user_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./templates/create_user.html")?)
}

// Fonction pour afficher la page HTML de connexion
async fn login_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./templates/login.html")?)
}

// Fonction pour afficher la page d'accueil personnalisée
async fn home_page(user_id: web::Path<String>) -> actix_web::Result<NamedFile> {
    // Construire le chemin vers le fichier HTML en fonction de l'ID de l'utilisateur
    let file_path = format!("./templates/home_{}.html", user_id);

    // Ouvrir et servir le fichier HTML
    Ok(NamedFile::open(file_path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    println!("Starting server on port {}", port);

    HttpServer::new(move || {
        App::new()
            // Routes pour l'affichage HTML
            .route("/create-user", web::get().to(create_user_page)) // Affiche la page de création d'utilisateur
            .route("/login", web::get().to(login_page)) // Affiche la page de connexion
            .route("/home/{user_id}", web::get().to(home_page))
            // Routes pour les appels API POST
            .route("/create-user", web::post().to(create_user_query)) // Gère le formulaire de création
            .route("/login", web::post().to(login_user_query)) // Gère le formulaire de connexion
            // Fichiers statiques
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .service(actix_files::Files::new("/", "./templates").index_file("index.html"))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .workers(2)
    .run()
    .await
}
