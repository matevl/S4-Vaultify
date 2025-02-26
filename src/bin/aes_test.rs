use s4_vaultify::backend::aes_keys::decrypted_key::*;
use s4_vaultify::backend::aes_keys::keys_password::*;
use s4_vaultify::backend::aes_keys::crypted_key::*;

use std::env;
use std::fs;
use std::process;

fn main() {
    // 1. Generate the AES-256 key from the login and password
    let login = "je_mappelle158965";
    let mdp = "jetestlesres";

    // Generate a salt from the login
    let salt = generate_salt_from_login(login);
    // Set the number of iterations for PBKDF2 (example: 100,000)
    let iterations = 100_000;
    // Derive the key using the password, salt, and iteration count
    let key = derive_key(mdp, &salt, iterations);
    // Display the key in hexadecimal format
    display_key_hex(&key);

    // 2. Define the path of the file to be encrypted (set directly here)
    let chemin_fichier = "assets/aes_test/video_test.bin"; // change this path as needed

    // Read the file
    let donnees = fs::read(chemin_fichier)
        .expect("Error reading the file");

    // 3. Encrypt the data
    let donnees_chiffrees = encrypt(&donnees, &key);
    // Save the encrypted file with the ".enc" extension
    let chemin_chiffre = format!("{}.enc", chemin_fichier);
    fs::write(&chemin_chiffre, &donnees_chiffrees)
        .expect("Error writing the encrypted file");
    println!("Encrypted file saved as: {}", chemin_chiffre);

    // 4. Decrypt the data
    match decrypt(&donnees_chiffrees, &key) {
        Ok(donnees_dechiffrees) => {
            // Save the decrypted file with the ".dec" extension
            let chemin_dechiffre = format!("{}.dec", chemin_fichier);
            fs::write(&chemin_dechiffre, &donnees_dechiffrees)
                .expect("Error writing the decrypted file");
            println!("Decrypted file saved as: {}", chemin_dechiffre);
        },
        Err(err) => {
            eprintln!("Error during decryption: {}", err);
            process::exit(1);
        }
    }
}
