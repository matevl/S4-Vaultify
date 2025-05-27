//use tokio_rustls::rustls::HandshakeType::Certificate;
use actix_files::Files;
use actix_files::NamedFile;
use actix_web::http::header;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use askama::Template;
use rusqlite::Result;
use rustls::Certificate;
use rustls::PrivateKey;
use rustls_pemfile::{certs, pkcs8_private_keys};
use s4_vaultify::backend::server_manager::account_manager::{
    create_user_query, get_user_vaults, login_user_query, logout_user_query, verify_code_query,
    CreateUserForm, JWT,
};
use s4_vaultify::backend::server_manager::file_manager::file_handler::{
    create_folder_query, download_file_query, get_file_tree_query, remove_file_query,
    remove_folder_query, rename_item_query, upload_file_query,
};
use s4_vaultify::backend::server_manager::global_manager::{
    init_server_config, CONNECTION, SESSION_CACHE,
};
use s4_vaultify::backend::server_manager::vault_manager::{
    create_vault_query, delete_vault_query, load_vault_query, share_vault_query, VaultInfo,
};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::Mutex;
use tera::Context;
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
fn get_user_from_cookie(req: &HttpRequest) -> Option<JWT> {
    req.cookie("user_token")
        .and_then(|cookie| serde_json::from_str(cookie.value()).ok())
}

// Create user
async fn create_user(form: web::Json<CreateUserForm>) -> HttpResponse {
    create_user_query(web::Json(form.into_inner())).await
}

// Main route

pub async fn home(req: HttpRequest) -> impl Responder {
    if let Some(decoded_jwt) = get_user_from_cookie(&req) {
        if SESSION_CACHE.get(&decoded_jwt.session_id).is_some() {
            let html = HomeTemplate {
                username: decoded_jwt.email.clone(),
                email: decoded_jwt.email.clone(),
                vault_info: decoded_jwt
                    .loaded_vault
                    .as_ref()
                    .map_or("No data".to_string(), |v| v.name.clone()),
            };
            return HttpResponse::Ok()
                .content_type("text/html")
                .body(html.render().unwrap());
        }
    }

    HttpResponse::Found()
        .insert_header((header::LOCATION, "/login"))
        .finish()
}

/**
 * Endpoint to fetch vaults for a user.
 *
 * @param req - The HTTP request (to extract session info).
 * @return An HTTP response containing the user's vaults or an error message.
 */
pub async fn get_user_vaults_query(req: HttpRequest) -> impl Responder {
    let jwt = match get_user_from_cookie(&req) {
        Some(jwt) => jwt,
        None => return HttpResponse::Found().finish(),
    };

    let con = CONNECTION.lock().unwrap();
    let vaults = match get_user_vaults(&con, jwt.id) {
        Ok(vaults) => vaults,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

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

    // Start Actix Web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone())) // Inject Tera into handlers
            .app_data(web::Data::new(Mutex::new(Vec::<String>::new()))) // Example of shared session, adapt as needed
            // GET routesding file tree.
            .route("/create-user", web::get().to(create_user_page))
            .route("/login", web::get().to(login_page))
            .route("/home", web::get().to(home))
            .route("/vaults", web::get().to(get_user_vaults_query))
            .route("/vaults/{vault_id}", web::get().to(vault_detail_page))
            .route(
                "/vaults/{vault_id}/tree",
                web::get().to(get_file_tree_query),
            )
            // POST routes
            .route("/create-user", web::post().to(create_user))
            .route("/login", web::post().to(login_user_query))
            .route("/logout", web::post().to(logout_user_query))
            .route("/create-vault", web::post().to(create_vault_query))
            .route("/load-vault", web::post().to(load_vault_query))
            .route("/delete-vault", web::post().to(delete_vault_query))
            .route(
                "/vaults/{vault_id}/tree",
                web::post().to(get_file_tree_query),
            )
            .route("/share-vault", web::post().to(share_vault_query))
            .route(
                "/vaults/{vault_id}/create-folder",
                web::post().to(create_folder_query),
            )
            .route(
                "/vaults/{vault_id}/rename-item",
                web::post().to(rename_item_query),
            )
            .route(
                "/vaults/{vault_id}/remove-folder",
                web::post().to(remove_folder_query),
            )
            .route(
                "/vaults/{vault_id}/remove-file",
                web::post().to(remove_file_query),
            )
            .route(
                "/vaults/{vault_id}/upload",
                web::post().to(upload_file_query),
            )
            .route(
                "/vaults/{vault_id}/download",
                web::post().to(download_file_query),
            )
            .route("/verify-code", web::post().to(verify_code_query))
            // Routes for static files (images, CSS, JS, etc.)
            .service(Files::new("/static", "../static").show_files_listing()) // Serve static content
            .service(Files::new("/", "../templates").index_file("index.html"))
    })
    .bind_rustls("0.0.0.0:443", Arc::try_unwrap(rustls_config).unwrap())? // Use SSL with Rustls
    .workers(8) // Number of workers (threads) to improve performance
    .run()
    .await
}
