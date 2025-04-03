use actix_files::NamedFile;
use actix_web::{web, App, HttpServer};
use s4_vaultify::backend::account_manager::account_server::{
    create_user_query, create_vault_query, get_vaults_list_query, init_db_connection,
    init_server_config, load_vault_query, login_user_query,
};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize)]
struct User {
    username: String,
    email: String,
}

// Fonction pour afficher la page HTML de création d'utilisateur
async fn create_user_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./templates/create_user.html")?)
}
async fn home_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./templates/home.html")?) // Assure-toi que tu as un fichier `home.html`
}
// Fonction pour afficher la page HTML de connexion
async fn login_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./templates/login.html")?)
}

// Fonction pour récupérer les informations de l'utilisateur depuis le cookie
fn get_user_from_cookie(req: &HttpRequest) -> Option<User> {
    if let Some(cookie) = req.cookie("user_token") {
        // Ici, tu dois récupérer les informations utilisateur à partir du token, ex : décryptage
        // Pour simplifier, ici on fait juste un exemple avec un utilisateur fictif
        Some(User {
            username: "JohnDoe".to_string(),
            email: "john.doe@example.com".to_string(),
        })
    } else {
        None
    }
}

// Fonction pour la page d'accueil, affiche un message personnalisé avec l'utilisateur

// Fonction pour traiter la connexion
async fn login_user(form: web::Json<LoginForm>, req: HttpRequest) -> HttpResponse {
    // Simuler une connexion réussie (ici tu pourrais vérifier avec une base de données)
    if form.username == "JohnDoe" && form.password == "password123" {
        let token = "fake_token_for_johndoe"; // Remplace par un vrai token généré
        let cookie = Cookie::build("user_token", token)
            .path("/")
            .max_age(Duration::days(7))  // Durée de validité du cookie
            .secure(false)  // À mettre à true en production (https)
            .http_only(true)  // Sécuriser le cookie
            .finish();

        // Réponse avec le cookie pour l'utilisateur
        HttpResponse::Ok()
            .cookie(cookie)
            .body("Connexion réussie!")
    } else {
        HttpResponse::Unauthorized().body("Nom d'utilisateur ou mot de passe incorrect.")
    }
}

// Fonction pour traiter l'enregistrement de l'utilisateur
async fn create_user(form: web::Json<CreateUserForm>) -> HttpResponse {
    // Simuler la création de l'utilisateur (vérifie les informations ici et ajoute-les à ta base de données)
    if form.username == "JohnDoe" && form.password == "password123" {
        HttpResponse::Ok().body("Utilisateur créé avec succès!")
    } else {
        HttpResponse::BadRequest().body("Erreur lors de la création de l'utilisateur.")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    println!("Starting server on port {}", port);

    init_server_config();

    HttpServer::new(move || {
        App::new()
            // Routes pour l'affichage HTML
            .route("/create-user", web::get().to(create_user_page))
            .route("/login", web::get().to(login_page))
            .route("/home", web::get().to(home_page))  // La route pour la page d'accueil
            // Routes pour les appels API POST
            .route("/create-user", web::post().to(create_user_query))
            .route("/login", web::post().to(login_user))
            .route("/vaults", web::post().to(get_vaults_list_query))
            .route("/create-vault", web::post().to(create_vault_query))
            .route("/load-vault", web::post().to(load_vault_query))
            // Fichiers statiques
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .service(actix_files::Files::new("/", "./templates").index_file("index.html"))
    })
        .bind(format!("0.0.0.0:{}", port))?
        .workers(8)
        .run()
        .await
}
