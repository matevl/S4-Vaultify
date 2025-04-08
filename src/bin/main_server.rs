use actix_files::NamedFile;
use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Responder, cookie::Cookie};
use s4_vaultify::backend::account_manager::account_server::{create_vault_query,
    create_user_query,  get_vaults_list_query, init_db_connection,
    init_server_config, load_vault_query, login_user_query, JWT
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::sync::Arc;
use tera::Context;
//use tokio_rustls::rustls::HandshakeType::Certificate;
use rustls::PrivateKey;
use s4_vaultify::backend::account_manager::account_server::CreateUserForm;
use tokio_rustls::rustls::ServerConfig;
use rustls::Certificate;
use askama::Template;
use s4_vaultify::backend::account_manager::account_server::get_user_vaults;
//use s4_vaultify::bin::main_server::CONNECTION;
use lazy_static::lazy_static;
use rusqlite::{Connection , Result};
use std::sync::Mutex;
use tera::Tera;
use reqwest::Body;
use std::path::Path;
use std::fs::read_to_string;
use s4_vaultify::backend::account_manager::account_server::create_vault;
use std::fs;
use s4_vaultify::backend::file_manager::mapping::init_map;
use s4_vaultify::backend::account_manager::account_server::Perms;
use s4_vaultify::backend::aes_keys::keys_password::generate_salt_from_login;
use s4_vaultify::backend::aes_keys::keys_password::derive_key;
use s4_vaultify::backend::aes_keys::keys_password::generate_random_key;
use s4_vaultify::backend::account_manager::account_server::VaultInfo;
use s4_vaultify::backend::VAULTS_DATA;
use std::time::UNIX_EPOCH;
use std::time::SystemTime;
use actix_files::Files;






lazy_static! {
    pub static ref CONNECTION: Mutex<Connection> =
        Mutex::new(Connection::open("my_db.sqlite").unwrap());
}
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

/*pub async fn vaults_page(req: HttpRequest) -> impl Responder {
    if let Some(token) = get_user_from_cookie(&req) {
        let secret = "test";
        if let Some(decoded_jwt) = JWT::decode(&token, secret) {
            let jwt_payload = web::Json(decoded_jwt);
            // Assure-toi que la réponse est un Box<dyn Responder> avec le type Body = String
            Box::new(get_vaults_list_query(jwt_payload).await) as Box<dyn Responder<Body = String>>
        } else {
            // Retourne une réponse avec le corps type String
            Box::new(HttpResponse::Unauthorized().body("Token invalide.")) as Box<dyn Responder<Body = String>>
        }
    } else {
        // Retourne une autre réponse avec le corps type String
        Box::new(HttpResponse::Unauthorized().body("Non authentifié.")) as Box<dyn Responder<Body = String>>
    }
}*/
/**
 * Endpoint to fetch vaults for a user.
 *
 * @param req - The HTTP request (to extract session info).
 * @return An HTTP response containing the user's vaults or an error message.
 */
pub async fn get_user_vaults_query(req: HttpRequest) -> impl Responder {
    // Récupérer le cookie JWT de la requête
    if let Some(token) = get_user_from_cookie(&req) {
        let secret = "test";

        // Décoder le token JWT
        match JWT::decode(&token, secret) {
            Some(decoded_jwt) => {
                // Charger les vaults depuis la base de données
                let conn = CONNECTION.lock().unwrap();
                match get_user_vaults(&conn, decoded_jwt.id) {
                    Ok(vaults) => {
                        // Lire le fichier HTML
                        let file_path = Path::new("templates/vaults.html");
                        let html_content = match read_to_string(file_path) {
                            Ok(content) => content,
                            Err(_) => {
                                return HttpResponse::InternalServerError()
                                    .body("Erreur lors du chargement du fichier HTML.");
                            }
                        };

                        // Convertir les vaults en une chaîne HTML
                        let vaults_html = vaults
                            .iter()
                            .map(|vault| {
                                format!(
                                    "<li><strong>{}</strong> (Utilisateur ID: {})<br><small>Date : {}</small></li>",
                                    vault.name, vault.user_id, vault.date
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n");

                        // Injecter les données dans le contenu HTML
                        let html = html_content
                            .replace("{{ username }}", &decoded_jwt.email)
                            .replace("{% for vault in vaults %}{% endfor %}", &vaults_html);

                        // Retourner le fichier HTML modifié
                        HttpResponse::Ok().content_type("text/html").body(html)
                    }
                    Err(e) => {
                        // Ici on renvoie l'erreur détaillée pour mieux comprendre le problème
                        return HttpResponse::InternalServerError().body(format!("Erreur lors de la récupération des vaults : {:?}", e));
                    }
                }
            }
            None => {
                // Si le token est invalide, renvoyer une erreur
                HttpResponse::Unauthorized().body("Token invalide ou expiré.")
            }
        }
    } else {
        // Si aucun token n'est trouvé, renvoyer une erreur
        HttpResponse::Unauthorized().body("Token manquant.")
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 443;
    println!("Démarrage du serveur sur le port {}", port);

    // Initialiser la configuration du serveur (si nécessaire)
    init_server_config();

    // Charger les fichiers de certificats pour SSL
    let cert_path = "../certs/certificate.crt"; // Vérifie l'emplacement des fichiers
    let key_path = "../certs/private_unencrypted.key";

    // Charger la configuration Rustls
    let rustls_config = load_rustls_config(cert_path, key_path);

    // Initialiser Tera pour gérer les templates
    let tera = Tera::new("templates/**/*").unwrap();  // Charger les templates de la directory

    // Démarrer le serveur Actix Web
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))  // Injecter Tera dans les handlers
            .app_data(web::Data::new(Mutex::new(Vec::<String>::new())))  // Exemple de session partagée, adapte comme nécessaire

            // Routes GET
            .route("/create-user", web::get().to(create_user_page))
            .route("/login", web::get().to(login_page))
            .route("/home", web::get().to(home))
            .route("/vaults", web::get().to(get_user_vaults_query))

            // Routes POST
            .route("/create-user", web::post().to(create_user))
            .route("/login", web::post().to(login_user_query))
            .route("/create-vault", web::post().to(create_vault_query))
            .route("/load-vault", web::post().to(load_vault_query))

            // Routes pour les fichiers statiques (images, CSS, JS, etc.)
            .service(Files::new("/static", "../static").show_files_listing())  // Servir le contenu statique
            .service(Files::new("/", "../templates").index_file("index.html"))  // Servir les templates, avec un fichier par défaut
    })
        .bind_rustls("0.0.0.0:443", Arc::try_unwrap(rustls_config).unwrap())?  // Utiliser SSL avec Rustls
        .workers(8)  // Nombre de workers (threads) pour améliorer les performances
        .run()
        .await
}