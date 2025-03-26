fn main() {
    match s4_vaultify::backend::auth::email::final_send("paul.boucheret@epita.fr") {
        Ok(info) => {
            println!("Code sent to {}", info.email);
            println!("Code: {}", info.code);
            match info.timestamp() {
                Some(ts) => println!("Sent at (UNIX timestamp): {}", ts),
                None => println!("Could not get send time"),
            }
        },
        Err(e) => eprintln!("Failed to send email: {}", e),
    }
}
