// main.rs

// Importation des modules depuis vos fichiers sources
use s4_vaultify::backend::aes_keys::decrypted_key::*;
use s4_vaultify::backend::aes_keys::keys_password::*;
use s4_vaultify::backend::aes_keys::crypted_key::*;

use std::env;
use std::fs;
use std::process;
fn main() {
    // 1. Génération de la clé AES-256 à partir du login et du mot de passe
    let login = "je_mappelle158965";
    let mdp = "jetestlesres";

    // Génération d'un salt à partir du login
    let salt = generate_salt_from_login(login);
    // Nombre d'itérations pour PBKDF2 (exemple : 100 000)
    let iterations = 100_000;
    // Dérivation de la clé à partir du mot de passe, du salt et du nombre d'itérations
    let key = derive_key(mdp, &salt, iterations);
    // Affichage de la clé en hexadécimal
    display_key_hex(&key);

    // 2. Chemin du fichier à chiffrer (défini directement ici)
    let chemin_fichier = "/home/specsaiko/Bureau/S4-Vaultify/assets/aes_test/video_test.bin"; // modifiez ce chemin selon vos besoins

    // Lecture du fichier
    let donnees = fs::read(chemin_fichier)
        .expect("Erreur lors de la lecture du fichier");

    // 3. Chiffrement des données
    let donnees_chiffrees = encrypt(&donnees, &key);
    // Sauvegarde du fichier chiffré avec l'extension ".enc"
    let chemin_chiffre = format!("{}.enc", chemin_fichier);
    fs::write(&chemin_chiffre, &donnees_chiffrees)
        .expect("Erreur lors de l'écriture du fichier chiffré");
    println!("Fichier chiffré sauvegardé sous : {}", chemin_chiffre);

    // 4. Déchiffrement des données
    match decrypt(&donnees_chiffrees, &key) {
        Ok(donnees_dechiffrees) => {
            // Sauvegarde du fichier déchiffré avec l'extension ".dec"
            let chemin_dechiffre = format!("{}.dec", chemin_fichier);
            fs::write(&chemin_dechiffre, &donnees_dechiffrees)
                .expect("Erreur lors de l'écriture du fichier déchiffré");
            println!("Fichier déchiffré sauvegardé sous : {}", chemin_dechiffre);
        },
        Err(err) => {
            eprintln!("Erreur lors du déchiffrement : {}", err);
            process::exit(1);
        }
    }
}