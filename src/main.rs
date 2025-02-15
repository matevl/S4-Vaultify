mod backend;
use s4_vaultify::backend::aes_keys::keys_password::*;
fn main() {
    let password = "test";
    let salt = b"un_sel_aleatoire_de_16_octets";
    let iterations = 100_000;

    let key = derive_key(password, salt, iterations);


    display_key_hex(&key);
}
