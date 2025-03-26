fn main() {
    match s4_vaultify::backend::auth::email::final_send("paul.boucheret@epita.fr") {
        Ok(info) => {
            println!("Code envoyé à {}", info.email);
            println!("Code: {}", info.code);
            println!("Envoyé à : {}", info.time.format("%Y-%m-%d %H:%M:%S"));
        }
        Err(e) => eprintln!("Erreur lors de l’envoi : {}", e),
    }
}
