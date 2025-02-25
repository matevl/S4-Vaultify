use ring::rand::SecureRandom;
use ring::rand::SystemRandom;
use std::fs::File;
use std::io::{Read, Write};

use s4_vaultify::backend::aes_keys::crypted_key::encrypt;
use s4_vaultify::backend::aes_keys::decrypted_key::decrypt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate a random key
    let key = generate_random_key();
    println!("Generated Key: {:?}", key);

    // File paths
    let input_file_path = "path/to/your/input.bin";
    let encrypted_file_path = "path/to/your/encrypted.bin";
    let decrypted_file_path = "path/to/your/decrypted.bin";

    // Read the input file
    let mut input_file = File::open(input_file_path)?;
    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data)?;

    // Encrypt the data
    let encrypted_data = encrypt(&input_data, &key);

    // Write the encrypted data to a file
    let mut encrypted_file = File::create(encrypted_file_path)?;
    encrypted_file.write_all(&encrypted_data)?;
    println!("Encrypted data written to {}", encrypted_file_path);

    // Read the encrypted file
    let mut encrypted_file = File::open(encrypted_file_path)?;
    let mut encrypted_data = Vec::new();
    encrypted_file.read_to_end(&mut encrypted_data)?;

    // Decrypt the data
    let decrypted_data = decrypt(&encrypted_data, &key)?;

    // Write the decrypted data to a file
    let mut decrypted_file = File::create(decrypted_file_path)?;
    decrypted_file.write_all(&decrypted_data)?;
    println!("Decrypted data written to {}", decrypted_file_path);

    Ok(())
}

// Function to generate a random key
pub fn generate_random_key() -> Vec<u8> {
    let rng = SystemRandom::new();
    let mut key = vec![0; 32];
    rng.fill(&mut key).expect("Failed to generate random key");
    key
}

// Assume the previous AES-related functions are included here...
