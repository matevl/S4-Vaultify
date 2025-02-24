use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use lazy_static::lazy_static;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use spin::Mutex;
use std::collections::HashMap;

use std::ops::Deref;
use regex::Regex;
use s4_vaultify::backend::account_manager::account::{JWT, UserData, UserInput, load_users_data, load_vault_matching, log_to_vaultify};
use s4_vaultify::backend::vault_manager::init_config_vaultify;

// Déclaration des variables globales
lazy_static! {
    static ref USERD: Mutex<Vec<UserData>> = Mutex::new(load_users_data("~/.vaultify/"));
    static ref VAULT_MATCh: Mutex<HashMap<String, Vec<(String, String)>>> = Mutex::new(load_vault_matching());
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct Choice {
    choice: String,
}

async fn homepage(choice: web::Query<Choice>) -> impl Responder {
    println!("Bienvenue !\n1. S'enregistrer\n2. Se connecter");

    match choice.choice.as_str() {
        "1" => {
            // L'utilisateur veut s'enregistrer, appeler la fonction d'enregistrement
            register_user().await
        }
        "2" => {
            // L'utilisateur veut se connecter, appeler la fonction de connexion
            login_user().await
        }
        _ => {
            // Si le choix est invalide
            HttpResponse::BadRequest().body("Choix invalide, veuillez réessayer.")
        }
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_config_vaultify();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default()) // Middleware pour logger les requêtes
            .route("/", web::get().to(homepage)) // Route principale
            .service(web::scope("/auth") // Groupe de routes d'authentification
                .route("/register", web::post().to(register_user))
                .route("/login", web::post().to(login_user))
            )
            .route("/account", web::get().to(account_details)) // Route pour afficher les détails du compte
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

// Fonction pour afficher la page d'accueil



// Fonction pour enregistrer un utilisateur
// Fonction d'enregistrement de l'utilisateur
async fn register_user() -> impl Responder {
    // Demander à l'utilisateur son email
    print!("Entrez votre email : ");
    io::stdout().flush().unwrap(); // Flush pour s'assurer que le message est bien affiché

    let mut email = String::new();
    io::stdin()
        .read_line(&mut email)
        .expect("Erreur lors de la lecture de l'email");
    if is_valid_email(&email) {
        // Supprimer les espaces ou les sauts de ligne de l'email
         email = email.trim().to_string();
    }
    else { 
        
    }



    // Demander à l'utilisateur son mot de passe
    print!("Entrez votre mot de passe : ");
    io::stdout().flush().unwrap();

    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .expect("Erreur lors de la lecture du mot de passe");

    // Supprimer les espaces ou les sauts de ligne du mot de passe
    let password = password.trim().to_string();

    // Créer un utilisateur avec les informations saisies
    let user_input = UserInput {
        email,
        password,
    };

    // Logique d'enregistrement : ici tu pourrais ajouter l'utilisateur à une base de données ou un fichier
    println!("Utilisateur enregistré : {:?}", user_input);
    
    // Retourner une réponse HTTP après l'enregistrement
    HttpResponse::Ok().json("Utilisateur enregistré avec succès")
}

// Fonction pour connecter un utilisateur
async fn login_user() -> impl Responder {
    print!("Entrez votre email : ");
    io::stdout().flush().unwrap(); // Flush pour s'assurer que le message est bien affiché

    let mut email = String::new();
    io::stdin()
        .read_line(&mut email)
        .expect("Erreur lors de la lecture de l'email");
    if is_valid_email(&email) {
        // Supprimer les espaces ou les sauts de ligne de l'email
        email = email.trim().to_string();
    }


    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .expect("Erreur lors de la lecture du mot de passe");

    // Supprimer les espaces ou les sauts de ligne du mot de passe
    let password = password.trim().to_string();

    // Créer un utilisateur avec les informations saisies
    let user_input = UserInput::new(email, password);

    let jwt = log_to_vaultify(&user_input, USERD.lock().deref()).expect("Erreur"); 
    match jwt {
        Ok(jwt) => {
            
        }
        None => {}
    }
    HttpResponse::Ok().json(jwt)
}

// Fonction pour afficher les informations de l'utilisateur
async fn account_details() -> impl Responder {
    // Supposons que le JWT est dans la session ou dans un token passé via l'en-tête Authorization
    HttpResponse::Ok().body("Détails du compte utilisateur")
}
fn is_valid_email(email: &str) -> bool {
    // Expression régulière pour valider l'email
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}