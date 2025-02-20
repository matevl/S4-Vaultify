mod backend;
mod error_manager;

use crate::backend::aes_keys::decrypted_key::{decrypt_block, pkcs7_unpad};
use s4_vaultify::backend::aes_keys::crypted_key::*;
use s4_vaultify::backend::aes_keys::keys_password::*;
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // On attend 5 arguments :
    // input_file (plaintext), encrypted_file, decrypted_file, password et login.
    let (input_file, encrypted_file, decrypted_file, password, login) = if args.len() >= 6 {
        (
            args[1].clone(),
            args[2].clone(),
            args[3].clone(),
            args[4].clone(),
            args[5].clone(),
        )
    } else {
        println!("Arguments insuffisants, utilisation des valeurs par défaut.");
        (
            "/home/specsaiko/Bureau/S4-Vaultify/assets/bin_1981022678.bin".to_string(),
            "/home/specsaiko/Bureau/S4-Vaultify/encrypted.bin".to_string(),
            "decrypted.bin".to_string(),
            "mon_mot_de_passe".to_string(),
            "lla686!!!!".to_string(),
        )
    };

    // --- CHIFFREMENT ---

    // 1. Lecture du fichier en clair
    let mut plaintext = fs::read(&input_file)?;
    println!(
        "Lecture du fichier {} ({} octets)",
        input_file,
        plaintext.len()
    );

    // 2. Ajout du padding PKCS#7 pour obtenir des blocs de 16 octets
    pkcs7_pad(&mut plaintext, 16);
    println!("Taille après padding: {} octets", plaintext.len());

    // 3. Dérivation de la clé à partir du password et du login (génération du sel)
    let salt = generate_salt_from_login(&login);
    let iterations = 100_000;
    let key = derive_key(&password, &salt, iterations);
    display_key_hex(&key);

    // 4. Expansion de la clé pour AES-256
    let round_keys = key_expansion(&key);

    // 5. Chiffrement bloc par bloc
    let mut ciphertext = Vec::with_capacity(plaintext.len());
    for block in plaintext.chunks(16) {
        let encrypted_block = encrypt_block(block, &round_keys);
        ciphertext.extend_from_slice(&encrypted_block);
    }
    fs::write(&encrypted_file, &ciphertext)?;
    println!("Fichier chiffré écrit dans {}", encrypted_file);

    // --- DÉCHIFFREMENT ---

    // Vérification de la taille (doit être multiple de 16)
    if ciphertext.len() % 16 != 0 {
        return Err("Le fichier chiffré n'est pas un multiple de 16 octets.".into());
    }

    // Déchiffrement bloc par bloc
    let mut decrypted_data = Vec::with_capacity(ciphertext.len());
    for block in ciphertext.chunks(16) {
        let decrypted_block = decrypt_block(block, &round_keys);
        decrypted_data.extend_from_slice(&decrypted_block);
    }

    // Suppression du padding PKCS#7
    pkcs7_unpad(&mut decrypted_data).map_err(|e| format!("Erreur lors du dépadding: {}", e))?;

    fs::write(&decrypted_file, &decrypted_data)?;
    println!("Fichier déchiffré écrit dans {}", decrypted_file);

    Ok(())
}
