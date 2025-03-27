use actix_files::NamedFile;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use s4_vaultify::backend::account_manager::account_server::{
    create_user_query, login_user_query, get_vaults_list_query, create_vault_query, load_vault_query, init_server_config, init_db_connection
};
use s4_vaultify::backend::{VAULTIFY_CONFIG, VAULTIFY_DATABASE, VAULTS_DATA};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use lazy_static::lazy_static;
use std::env;

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

lazy_static! {
    /**
     * Root directory path for the application.
     */
    pub static ref ROOT: std::path::PathBuf = dirs::home_dir().expect("Could not find home dir");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    println!("Starting server on port {}", port);

    // Construct the path for the database
    let database_path = ROOT.join(VAULTIFY_DATABASE);

    // Initialize the server configuration
    init_server_config(&ROOT).await;

    // Initialize the connection to the database
    let conn = init_db_connection(database_path.to_str().unwrap()).unwrap();

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            // Add the connection to the application context
            .app_data(web::Data::new(conn.clone()))
            // Routes for displaying HTML
            .route("/create-user", web::get().to(create_user_page)) // Displays the user creation page
            .route("/login", web::get().to(login_page)) // Displays the login page
            // Routes for API POST requests
            .route("/create-user", web::post().to(create_user_query)) // Handles the user creation form
            .route("/login", web::post().to(login_user_query)) // Handles the login form
            .route("/vaults", web::get().to(get_vaults_list_query)) // Handles the list of vaults for a user
            .route("/create-vault", web::post().to(create_vault_query)) // Handles the creation of a new vault
            .route("/load-vault", web::post().to(load_vault_query)) // Handles the loading of a vault
            // Static files
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .service(actix_files::Files::new("/", "./templates").index_file("index.html"))
    })
        .bind(format!("127.0.0.1:{}", port))?
        .workers(2)
        .run()
        .await
}
