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
use rusqlite::params;





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

/*pub fn add_file_to_db(conn: &Connection, vault_id: u32, name: &str, path: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO files (vault_id, name, path, uploaded_at) VALUES (?, ?, ?, ?)",
        params![vault_id, name, path, chrono::Utc::now().timestamp()],
    )?;
    Ok(())
}*/

/*async fn add_file_to_vault(
    vault_id: web::Path<u32>,
    mut payload: Multipart,
) -> HttpResponse {
    let vault_id = vault_id.into_inner();
    let upload_dir = format!("./uploads/vault_{}", vault_id);

    // Vérifier si le répertoire existe, sinon le créer
    if let Err(e) = fs::create_dir_all(&upload_dir) {
        eprintln!("Erreur lors de la création du répertoire : {:?}", e);
        return HttpResponse::InternalServerError().body("Erreur lors de la création du répertoire.");
    }

    // Parcourir les parties de la requête (fichiers)
    while let Ok(Some(mut field)) = payload.try_next().await {
        // Récupérer le nom du fichier
        let content_disposition = match field.content_disposition() {
            Some(cd) => cd,
            None => {
                eprintln!("Disposition du contenu manquante.");
                continue;
            }
        };

        let filename = match content_disposition.get_filename() {
            Some(name) => sanitize_filename::sanitize(name),
            None => {
                eprintln!("Nom du fichier manquant.");
                continue;
            }
        };

        let filepath = format!("{}/{}", upload_dir, filename);

        // Écrire le fichier sur le disque
        let mut f = match web::block(|| std::fs::File::create(&filepath)).await {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Erreur lors de la création du fichier : {:?}", e);
                return HttpResponse::InternalServerError().body("Erreur lors de l'écriture du fichier.");
            }
        };

        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Erreur lors de la lecture du chunk : {:?}", e);
                    return HttpResponse::InternalServerError().body("Erreur lors de la lecture des données.");
                }
            };

            if let Err(e) = web::block(move || f.write_all(&data).map(|_| f)).await {
                eprintln!("Erreur lors de l'écriture sur le disque : {:?}", e);
                return HttpResponse::InternalServerError().body("Erreur lors de l'écriture des données.");
            }
        }

        // Enregistrer l'information dans la base de données
        if let Err(e) = save_file_to_db(vault_id, &filename, &filepath).await {
            eprintln!("Erreur lors de l'insertion dans la base de données : {:?}", e);
            return HttpResponse::InternalServerError().body("Erreur lors de l'enregistrement en base.");
        }
    }

    HttpResponse::Ok().body("Fichier(s) ajouté(s) avec succès !")
}*/

/*async fn save_file_to_db(vault_id: u32, filename: &str, filepath: &str) -> Result<(), rusqlite::Error> {
    let conn = get_connection(); // À implémenter pour récupérer votre connexion
    conn.execute(
        "INSERT INTO files (vault_id, name, path, uploaded_at) VALUES (?, ?, ?, ?)",
        params![vault_id, filename, filepath, chrono::Utc::now().timestamp()],
    )?;
    Ok(())
}*/

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
        let secret = "test";  // Remplacer par votre clé secrète pour JWT

        // Décoder le token JWT
        match JWT::decode(&token, secret) {
            Some(decoded_jwt) => {
                // Appeler la fonction qui retourne un `impl Responder` (ici un `HttpResponse`)
                let vaults = get_vaults_list_query(web::Json(decoded_jwt)).await;

                // Créer un contexte pour Tera
                let mut context = Context::new();
                context.insert("vaults", &vaults);

                // Charger le template Tera
                let tera = Tera::new("templates/**/*").unwrap(); // Assurez-vous que les templates sont dans le dossier `templates`

                // Rendre le template avec les données
                let rendered_html = tera.render("vaults.html", &context).unwrap();

                // Retourner la réponse HTTP avec le contenu généré
                HttpResponse::Ok().content_type("text/html").body(rendered_html)
            },
            None => {
                // Retourner une réponse "Token invalide" ou toute autre erreur
                HttpResponse::Unauthorized().body("Token invalide ou expiré.")
            }
        }
    } else {
        // Si aucun token n'est trouvé, renvoyer une erreur
        HttpResponse::Unauthorized().body("Token manquant.")
    }
}
pub fn get_connection() -> Connection {
    let database_path = "./vaultify.db"; // Chemin configuré dans init_server_config
    Connection::open(database_path).expect("Erreur lors de la connexion à la base de données")
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
            //.route("/vault/{vault_id}/add-file", web::post().to(add_file_to_vault))
            // Routes pour les fichiers statiques (images, CSS, JS, etc.)
            .service(Files::new("/static", "../static").show_files_listing())  // Servir le contenu statique
            .service(Files::new("/", "../templates").index_file("index.html"))  // Servir les templates, avec un fichier par défaut
    })
        .bind_rustls("0.0.0.0:443", Arc::try_unwrap(rustls_config).unwrap())?  // Utiliser SSL avec Rustls
        .workers(8)  // Nombre de workers (threads) pour améliorer les performances
        .run()
        .await
}