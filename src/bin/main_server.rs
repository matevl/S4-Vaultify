use actix_files::NamedFile;
use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Responder, cookie::Cookie};
use s4_vaultify::backend::account_manager::account_server::{
    create_user_query, create_vault_query, get_vaults_list_query, init_db_connection,
    init_server_config, load_vault_query, login_user_query, JWT
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::sync::Arc;
//use tokio_rustls::rustls::HandshakeType::Certificate;
use rustls::PrivateKey;
use s4_vaultify::backend::account_manager::account_server::CreateUserForm;
use tokio_rustls::rustls::ServerConfig;
use rustls::Certificate;
use askama::Template;

fn load_rustls_config(cert_path: &str, key_path: &str) -> Arc<ServerConfig> {
    // Ouverture des fichiers de certificat et de clé privée
    let cert_file = File::open(cert_path).expect("Impossible d'ouvrir le fichier du certificat");
    let key_file = File::open(key_path).expect("Impossible d'ouvrir le fichier de clé privée");

    // Chargement de la chaîne de certificats
    let cert_chain: Vec<Certificate> = certs(&mut BufReader::new(cert_file))
        .map(|cert| cert.map(|cert_der| Certificate(cert_der.as_ref().to_vec())))
        .collect::<Result<Vec<_>, _>>()
        .expect("Erreur de chargement de la chaîne de certificats");

    // Chargement de la clé privée
    let key = pkcs8_private_keys(&mut BufReader::new(key_file))
        .collect::<Result<Vec<_>, _>>()
        .expect("Erreur de chargement de la clé privée")
        .into_iter()
        .next()
        .expect("Aucune clé privée trouvée");

    // Conversion de la clé privée en PrivateKey
    let private_key = PrivateKey(key.secret_pkcs8_der().to_vec());
    // Création de la configuration du serveur
    let config = ServerConfig::builder()
        .with_safe_defaults() // Utiliser des paramètres de sécurité par défaut pour la configuration
        .with_no_client_auth() // Désactiver l'authentification du client
        .with_single_cert(cert_chain, private_key) // Ajouter le certificat et la clé privée
        .expect("Impossible de configurer le certificat");

    Arc::new(config)
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    username: String,
    email: String,
    vault_info: String,
}

// Page d'inscription
async fn create_user_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("../templates/create_user.html")?)
}

// Page de connexion
async fn login_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("../templates/login.html")?)
}

// Récupérer l'utilisateur à partir du cookie
fn get_user_from_cookie(req: &HttpRequest) -> Option<String> {
    if let Some(cookie) = req.cookie("user_token") {
        Some(cookie.value().to_string()) // Retourner directement la valeur du token
    } else {
        None
    }
}

// Création d'utilisateur
async fn create_user(form: web::Json<CreateUserForm>) -> HttpResponse {
    create_user_query(web::Json(form.into_inner())).await


}

// Route principale
pub async fn home(req: HttpRequest) -> impl Responder {
    if let Some(token) = get_user_from_cookie(&req) {
        let secret = "test";

        match JWT::decode(&token, secret) {
            Some(decoded_jwt) => {
                let html = HomeTemplate {
                    username: decoded_jwt.email.clone(),
                    email: decoded_jwt.email.clone(),
                    vault_info: match &decoded_jwt.loaded_vault {
                        Some(vault) => vault.name.clone(), // ou vault.id, ou un champ que tu veux
                        None => "Aucune donnée".to_string(),
                    },
                };
                HttpResponse::Ok().content_type("text/html").body(html.render().unwrap())
            }
            None => HttpResponse::Unauthorized().body("Token invalide ou expiré."),
        }
    } else {
        HttpResponse::Unauthorized().body("Aucun token trouvé.")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 443;
    println!("Démarrage du serveur sur le port {}", port);

    init_server_config();

    let cert_path = "../certs/certificate.crt"; // Vérifie l'emplacement des fichiers
    let key_path = "../certs/private_unencrypted.key";

    let rustls_config = load_rustls_config(cert_path, key_path);
    HttpServer::new(move || {
        App::new()
            //recup HTML
            .route("/create-user", web::get().to(create_user_page))
            .route("/login", web::get().to(login_page))
            .route("/home", web::get().to(home))
            .route("/create-user", web::post().to(create_user))
            //recup post
            .route("/login", web::post().to(login_user_query))
            .route("/vaults", web::post().to(get_vaults_list_query))
            .route("/create-vault", web::post().to(create_vault_query))
            .route("/load-vault", web::post().to(load_vault_query))
            //recup templates
            .service(actix_files::Files::new("/static", "../static").show_files_listing())
            .service(actix_files::Files::new("/", "../templates").index_file("index.html"))
    })
        .bind_rustls("0.0.0.0:443", Arc::try_unwrap(rustls_config).unwrap())?

        .workers(8)
        .run()
        .await
}
