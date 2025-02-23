mod models;

use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use lazy_static::lazy_static;
use models::*;
use s4_vaultify::backend::*;
use serde::{Deserialize, Serialize};
use spin::Mutex;
use std::collections::HashMap;
use std::ops::Deref;

use account_manager::account::JWT;
// Structure pour les identifiants utilisateur
use account_manager::account::{
    UserData, UserInput, load_users_data, load_vault_matching, log_to_vaultify, save_vault_matching,
};
use s4_vaultify::backend::vault_manager::init_config_vaultify;

// Impor
lazy_static! {
    static ref USERD: Mutex<Vec<UserData>> = Mutex::new(load_users_data("~/.vaultify/"));
    static ref VAULT_MATCh: Mutex<HashMap<String, Vec<(String, String)>>> =
        Mutex::new(load_vault_matching());
}

pub fn homepage() {
    println!("Bienvenue !\n1. S'enregistrer\n2. Se connecter");
    let mut choix = String::new();
    std::io::stdin()
        .read_line(&mut choix)
        .expect("Erreur lors de la lecture du choix");

    match choix.trim() {
        "1" => {
            register_user("TEST".to_string(), "TEST".to_string(), "TEST".to_string());
        }
        "2" => {
            login_user();
        }
        _ => {
            println!("Choix invalide, veuillez réessayer.");
            homepage(); // Réappeler la fonction en cas de mauvais choix
        }
    }
}

fn register_user(email: String, password: String, name: String) {
    println!("Veuillez entrer vos informations pour vous enregistrer.");
    let mut username = String::new();
    let mut password = String::new();

    println!("Nom d'utilisateur : ");
    std::io::stdin()
        .read_line(&mut username)
        .expect("Erreur lors de la saisie du nom d'utilisateur");

    println!("Mot de passe : ");
    std::io::stdin()
        .read_line(&mut password)
        .expect("Erreur lors de la saisie du mot de passe");

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

    println!("Email : ");
    std::io::stdin()
        .read_line(&mut username)
        .expect("Erreur lors de la saisie du nom d'utilisateur");

    println!("Mot de passe : ");
    std::io::stdin()
        .read_line(&mut password)
        .expect("Erreur lors de la saisie du mot de passe");

    let user_input = UserInput::new(username.trim().to_string(), password.trim().to_string());

    // Utilise la fonction load_user_data de account.rs
    match log_to_vaultify(&user_input, USERD.lock().deref()) {
        Ok(jwt) => {
            println!("Connexion réussie ! Données utilisateur :");
            show_acc(&jwt);
        }
        Err(_) => {
            println!("Nom d'utilisateur ou mot de passe incorrect. Veuillez réessayer.");
            login_user();
        }
    }
}

fn show_acc(jwt: &JWT) {}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_config_vaultify();
    HttpServer::new(|| App::new())
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
