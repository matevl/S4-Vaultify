use actix_files::NamedFile;
use actix_web::{cookie::Cookie, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use rustls_pemfile::{certs, pkcs8_private_keys};
use s4_vaultify::backend::account_manager::account_server::{
    clean_expired_sessions, create_user_query, create_vault_query, get_vaults_list_query,
    init_server_config, load_vault_query, login_user_query, CreateUserForm, VaultInfo, JWT,
};
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::sync::Arc;
use tera::Context;
//use tokio_rustls::rustls::HandshakeType::Certificate;
use actix_files::Files;
use askama::Template;
use rusqlite::{Connection, Result};
use rustls::Certificate;
use rustls::PrivateKey;
use s4_vaultify::backend::file_manager::mapping::{get_tree_vault, move_tree_vault};
use std::sync::Mutex;
use tera::Tera;
use tokio_rustls::rustls::ServerConfig;

fn load_rustls_config(cert_path: &str, key_path: &str) -> Arc<ServerConfig> {
    // Open certificate and private key files
    let cert_file = File::open(cert_path).expect("Unable to open certificate file");
    let key_file = File::open(key_path).expect("Unable to open private key file");

    // Load certificate chain
    let cert_chain: Vec<Certificate> = certs(&mut BufReader::new(cert_file))
        .map(|cert| cert.map(|cert_der| Certificate(cert_der.as_ref().to_vec())))
        .collect::<Result<Vec<_>, _>>()
        .expect("Error loading certificate chain");

    // Load private key
    let key = pkcs8_private_keys(&mut BufReader::new(key_file))
        .collect::<Result<Vec<_>, _>>()
        .expect("Error loading private key")
        .into_iter()
        .next()
        .expect("No private key found");

    // Convert private key to PrivateKey
    let private_key = PrivateKey(key.secret_pkcs8_der().to_vec());
    // Create server configuration
    let config = ServerConfig::builder()
        .with_safe_defaults() // Use default security parameters for configuration
        .with_no_client_auth() // Disable client authentication
        .with_single_cert(cert_chain, private_key) // Add certificate and private key
        .expect("Unable to configure certificate");

    Arc::new(config)
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    username: String,
    email: String,
    vault_info: String,
}

// Sign-up page
async fn create_user_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("../templates/create_user.html")?)
}

// Login page
async fn login_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("../templates/login.html")?)
}

// Retrieve user from cookie
fn get_user_from_cookie(req: &HttpRequest) -> Option<String> {
    if let Some(cookie) = req.cookie("user_token") {
        Some(cookie.value().to_string()) // Directly return the token value
    } else {
        None
    }
}

// Create user
async fn create_user(form: web::Json<CreateUserForm>) -> HttpResponse {
    create_user_query(web::Json(form.into_inner())).await
}

// Main route
pub async fn home(req: HttpRequest) -> impl Responder {
    if let Some(token) = get_user_from_cookie(&req) {
        let secret = "test";

        match JWT::decode(&token, secret) {
            Some(decoded_jwt) => {
                let html = HomeTemplate {
                    username: decoded_jwt.email.clone(),
                    email: decoded_jwt.email.clone(),
                    vault_info: match &decoded_jwt.loaded_vault {
                        Some(vault) => vault.name.clone(), // or vault.id, or any field you want
                        None => "No data".to_string(),
                    },
                };
                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(html.render().unwrap())
            }
            None => HttpResponse::Unauthorized().body("Invalid or expired token."),
        }
    } else {
        HttpResponse::Unauthorized().body("No token found.")
    }
}

/**
 * Endpoint to fetch vaults for a user.
 *
 * @param req - The HTTP request (to extract session info).
 * @return An HTTP response containing the user's vaults or an error message.
 */
pub async fn get_user_vaults_query(req: HttpRequest) -> impl Responder {
    if let Some(token) = get_user_from_cookie(&req) {
        let secret = "test"; // Replace with your secret key for JWT

        match JWT::decode(&token, secret) {
            Some(decoded_jwt) => {
                let vaults_response = get_vaults_list_query(web::Json(decoded_jwt)).await;

                // Extract the body and deserialize into Vec<VaultInfo>
                let body = vaults_response.into_body();
                let body_bytes = actix_web::body::to_bytes(body).await.unwrap();
                let vaults: Vec<VaultInfo> = serde_json::from_slice(&body_bytes).unwrap();

                // Create the context for Tera
                let mut context = Context::new();
                context.insert("vaults", &vaults);

                // Load and render the template
                let tera = Tera::new("../templates/**/*").unwrap();
                let rendered_html = tera.render("vaults.html", &context).unwrap();

                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(rendered_html)
            }
            None => HttpResponse::Unauthorized().body("Invalid or expired token."),
        }
    } else {
        HttpResponse::Unauthorized().body("Missing token.")
    }
}

async fn vault_detail_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("../templates/vault_detail.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 443;
    println!("Starting server on port {}", port);

    // Initialize server configuration (if necessary)
    init_server_config();

    // Load certificate files for SSL
    let cert_path = "../certs/certificate.crt"; // Verify the file locations
    let key_path = "../certs/private_unencrypted.key";

    // Load Rustls configuration
    let rustls_config = load_rustls_config(cert_path, key_path);

    // Initialize Tera to handle templates
    let tera = Tera::new("../templates/**/*").unwrap(); // Load templates from the directory

    tokio::spawn(async {
        clean_expired_sessions().await;
    });

    // Start Actix Web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone())) // Inject Tera into handlers
            .app_data(web::Data::new(Mutex::new(Vec::<String>::new()))) // Example of shared session, adapt as needed
            // GET routes
            .route("/create-user", web::get().to(create_user_page))
            .route("/login", web::get().to(login_page))
            .route("/home", web::get().to(home))
            .route("/vaults", web::get().to(get_user_vaults_query))
            .route("/vaults/{vault_id}", web::get().to(vault_detail_page))
            // POST routes
            .route("/create-user", web::post().to(create_user))
            .route("/login", web::post().to(login_user_query))
            .route("/create-vault", web::post().to(create_vault_query))
            .route("/load-vault", web::post().to(load_vault_query))
            .route("/vaults/{vault_id}/tree", web::get().to(get_tree_vault))
            .route("/vaults/{vault_id}/move", web::post().to(move_tree_vault))
            //.route("/vault/{vault_id}/add-file", web::post().to(add_file_to_vault))
            // Routes for static files (images, CSS, JS, etc.)
            .service(Files::new("/static", "../static").show_files_listing()) // Serve static content
            .service(Files::new("/", "../templates").index_file("index.html")) // Serve templates, with a default file
    })
    .bind_rustls("0.0.0.0:443", Arc::try_unwrap(rustls_config).unwrap())? // Use SSL with Rustls
    .workers(8) // Number of workers (threads) to improve performance
    .run()
    .await
}
