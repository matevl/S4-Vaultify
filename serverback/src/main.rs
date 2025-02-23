mod models;


use actix_web::{web, App, HttpServer,HttpResponse, Responder};
use actix_web::middleware::Logger;
use models::*;
use serde::{Deserialize, Serialize};
use s4_vaultify::backend::*;


use account_manager::account::JWT;
// Structure pour les identifiants utilisateur
use account_manager::account::{UserInput, local_log_in}; // Impor


pub fn homepage() {
    println!("Bienvenue !\n1. S'enregistrer\n2. Se connecter");
    let mut choix = String::new();
    std::io::stdin().read_line(&mut choix).expect("Erreur lors de la lecture du choix");

    match choix.trim() {
        "1" => {
            register_user("TEST".to_string(),"TEST".to_string(),"TEST".to_string());
        },
        "2" => {
            login_user();
        },
        _ => {
            println!("Choix invalide, veuillez réessayer.");
            homepage(); // Réappeler la fonction en cas de mauvais choix
        }
    }
}

fn register_user(email: String, password: String, name: String ) {
    println!("Veuillez entrer vos informations pour vous enregistrer.");
    let mut username = String::new();
    let mut password = String::new();

    println!("Nom d'utilisateur : ");
    std::io::stdin().read_line(&mut username).expect("Erreur lors de la saisie du nom d'utilisateur");

    println!("Mot de passe : ");
    std::io::stdin().read_line(&mut password).expect("Erreur lors de la saisie du mot de passe");

    let user_input = UserInput {
        email: username.trim().to_string(),
        password: password.trim().to_string(),
    };

    // Ajoute une logique pour sauvegarder les informations utilisateur si nécessaire
    println!("Utilisateur enregistré avec succès : {:?}", user_input);
}

fn login_user() {
    println!("Veuillez entrer vos informations pour vous connecter.");
    let mut username = String::new();
    let mut password = String::new();

    println!("Nom d'utilisateur : ");
    std::io::stdin().read_line(&mut username).expect("Erreur lors de la saisie du nom d'utilisateur");

    println!("Mot de passe : ");
    std::io::stdin().read_line(&mut password).expect("Erreur lors de la saisie du mot de passe");

    let user_input = UserInput::new(username.trim().to_string(), password.trim().to_string());

    // Utilise la fonction load_user_data de account.rs
    match local_log_in(&user_input, ) {
        Some(user_data) => {
            println!("Connexion réussie ! Données utilisateur : {:?}", user_data);
        }
        None => {
            println!("Nom d'utilisateur ou mot de passe incorrect. Veuillez réessayer.");
            login_user();
        }
    }
}





#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/login", web::post().to(login))
            .route("/register", web::post().to(register))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}