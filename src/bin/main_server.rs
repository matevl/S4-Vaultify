use actix_files::Files;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use s4_vaultify::backend::account_manager::account_server::*;
// Gestion des formulaires (POST)
async fn handle_create_user(form: web::Form<CreateUserForm>) -> impl Responder {
    println!("Received form data: {:?}", form);
    HttpResponse::Ok().body("User created successfully!")
}

async fn handle_login(form: web::Form<LoginForm>) -> impl Responder {
    println!("Received form data: {:?}", form);
    HttpResponse::Ok().body("Login successful!")
}

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


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    println!("Starting server on port {}", port);

    HttpServer::new(move || {
        App::new()
            // Routes pour l'affichage HTML
            .route("/create-user", web::get().to(create_user_page))
            .route("/login", web::get().to(login_page))
            // Routes pour les appels API POST
            .route("/create-user", web::post().to(create_user_query))
            .route("/login", web::post().to(login_user_query))
            // Fichiers statiques
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .service(actix_files::Files::new("/", "./templates").index_file("index.html"))
    })
        .bind(format!("127.0.0.1:{}", port))?
        .workers(2)
        .run()
        .await
}


