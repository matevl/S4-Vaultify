// Importing the necessary libraries
use ring::pbkdf2; // For password-based key derivation via PBKDF2
use ring::rand::{SecureRandom, SystemRandom}; // For secure random number generation
use sha2::{Digest, Sha256}; // For SHA256 hashing
use std::num::NonZeroU32; // For working with non-zero integers

// Function that derives an AES-256 key from a password, a salt, and an iteration count
pub fn derive_key(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
    // Initialize a vector of 32 bytes (256 bits) to store the derived key
    let mut key = vec![0; 32];
    // Convert the iteration count to a NonZeroU32, producing an error if the value is zero
    let iterations = NonZeroU32::new(iterations).expect("error");
    // Apply the PBKDF2 key derivation function using HMAC-SHA256
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256, // Hash algorithm used by PBKDF2
        iterations,                 // Number of iterations (must be non-zero)
        salt,                       // Salt used in the derivation
        password.as_bytes(),        // Password converted to bytes
        &mut key,                   // Vector where the derived key will be stored
    );

    key // Return the derived key
}

// Function that displays the derived key in hexadecimal
pub fn display_key_hex(key: &[u8]) {
    // Convert the key into a hexadecimal string
    let hex_key: String = key.iter().map(|byte| format!("{:02X}", byte)).collect();
    // Print the derived AES-256 key
    println!("Derived AES-256 key: {}", hex_key);
}

// Function that generates a salt from a login using SHA256
pub fn generate_salt_from_login(login: &str) -> Vec<u8> {
    // Create a new SHA256 hasher
    let mut hasher = Sha256::new();
    // Update the hasher with the login
    hasher.update(login);
    // Compute the final hash
    let result = hasher.finalize();
    // Use the first 16 bytes of the hash as the salt
    let mut salt = result[..16].to_vec();
    // If the login is shorter than 16 characters, pad the salt with zeros
    if login.len() < 16 {
        salt.resize(16, 0);
    }

    salt // Return the generated salt
}

pub fn generate_random_key() -> Vec<u8> {
    // Create a new instance of the system random number generator
    let rng = SystemRandom::new();

    // Initialize a vector of 32 bytes (256 bits) to store the random key
    let mut key = vec![0; 32];

    // Fill the vector with random bytes using the secure random number generator
    rng.fill(&mut key).expect("Failed to generate random key");

    key // Return the generated random key
}
