mod backend;
mod error_manager;

use s4_vaultify::backend::aes_keys::keys_password::*;
fn main() {
    let password = "cheeese";
    let salt = b"azlkdvtrefdhytsr";
    let iterations = 100_000;

    let key = derive_key(password, salt, iterations);


    display_key_hex(&key);
}
