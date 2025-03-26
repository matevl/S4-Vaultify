

fn main() {
    let code = s4_vaultify::backend::auth::email::generate_code();
    println!("Generated code: {}", code);

    if let Err(e) = s4_vaultify::backend::auth::email::send_email("matteo.evola@epita.fr", &code) {
        eprintln!("Erreur lors de l’envoi du mail: {}", e);
    } else {
        println!("Code envoyé !");
    }
}