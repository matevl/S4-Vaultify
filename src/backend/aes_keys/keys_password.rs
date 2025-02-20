use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};
use std::num::NonZeroU32;

pub fn derive_key(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
    let mut key = vec![0; 32];
    let iterations = NonZeroU32::new(iterations).expect("erreur");
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        password.as_bytes(),
        &mut key,
    );

    key
}

pub fn display_key_hex(key: &[u8]) {
    let hex_key: String = key.iter().map(|byte| format!("{:02X}", byte)).collect();
    println!("Clé AES-256 dérivée: {}", hex_key);
}

pub fn generate_salt_from_login(login: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(login);
    let result = hasher.finalize();
    let mut salt = result[..16].to_vec();
    if login.len() < 16 {
        salt.resize(16, 0);
    }

    salt
}
