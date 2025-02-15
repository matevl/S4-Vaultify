use ring::pbkdf2;
use ring::rand::{SystemRandom, SecureRandom};
use data_encoding::HEXUPPER;
use std::num::NonZeroU32;

pub fn derive_key(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
    let mut key = vec![0; 32]; // AES-256 requires a 32-byte key
    let iterations = NonZeroU32::new(iterations).expect("Iterations must be nonzero");

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
    println!("Clé AES-256 dérivée: {}", HEXUPPER.encode(key));
}
