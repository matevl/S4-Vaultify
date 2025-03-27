use actix_files::NamedFile;
use actix_web::{web, App, HttpServer};
use s4_vaultify::backend::account_manager::account_server::{
    create_user_query, create_vault_query, get_vaults_list_query, init_db_connection,
    init_server_config, load_vault_query, login_user_query,
};

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    println!("Starting server on port {}", port);

    init_server_config();

    HttpServer::new(move || {
        App::new()
            .route("/create-user", web::get().to(create_user_page))
            .route("/login", web::get().to(login_page))
            .route("/create-user", web::post().to(create_user_query))
            .route("/login", web::post().to(login_user_query))
            .route("/vaults", web::get().to(get_vaults_list_query))
            .route("/create-vault", web::post().to(create_vault_query))
            .route("/load-vault", web::post().to(load_vault_query))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .service(actix_files::Files::new("/", "./templates").index_file("index.html"))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .workers(8)
    .run()
    .await
}
