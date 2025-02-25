use s4_vaultify::backend::account_manager::account::{
    add_user_to_data, load_users_data, log_to_vaultify, save_users_data, UserInput,
};
use s4_vaultify::backend::vault_manager::{init_config_vaultify, init_vault, VaultManager};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <command> [<args>]", args[0]);
        eprintln!("Commands:");
        eprintln!("  init-config");
        eprintln!("  init-vault <email> <password> <path>");
        eprintln!("  load-vault <email> <password> <path>");
        eprintln!("  add-user <email> <password>");
        eprintln!("  login <email> <password>");
        return;
    }

    match args[1].as_str() {
        "init-config" => {
            init_config_vaultify();
            println!("Configuration initialized.");
        }
        "init-vault" => {
            if args.len() != 5 {
                eprintln!("Usage: {} init-vault <email> <password> <path>", args[0]);
                return;
            }
            let email = &args[2];
            let password = &args[3];
            let path = &args[4];

            let users_data = load_users_data();
            let user_input = UserInput::new(email.clone(), password.clone());
            match log_to_vaultify(&user_input, &users_data) {
                Ok(mut jwt) => {
                    let mut vault_manager = VaultManager::new(vec![], path);
                    vault_manager.logged_users = Some(jwt);
                    if let Err(e) = init_vault(&mut vault_manager, path) {
                        eprintln!("Failed to initialize vault: {}", e);
                    } else {
                        println!("Vault initialized at {}", path);
                    }
                }
                Err(e) => eprintln!("Login failed: {}", e),
            }
        }
        "load-vault" => {
            if args.len() != 5 {
                eprintln!("Usage: {} load-vault <email> <password> <path>", args[0]);
                return;
            }
            let email = &args[2];
            let password = &args[3];
            let path = &args[4];

            let users_data = load_users_data();
            let user_input = UserInput::new(email.clone(), password.clone());
            match log_to_vaultify(&user_input, &users_data) {
                Ok(mut jwt) => {
                    let mut vault_manager = VaultManager::new(vec![], path);
                    vault_manager.logged_users = Some(jwt);
                    if let Err(e) = vault_manager.load_vault(path) {
                        eprintln!("Failed to load vault: {}", e);
                    } else {
                        println!("Vault loaded from {}", path);
                    }
                }
                Err(e) => eprintln!("Login failed: {}", e),
            }
        }
        "add-user" => {
            if args.len() != 4 {
                eprintln!("Usage: {} add-user <email> <password>", args[0]);
                return;
            }
            let email = &args[2];
            let password = &args[3];
            let mut users_data = load_users_data();
            let user_input = UserInput::new(email.clone(), password.clone());
            if let Err(e) = add_user_to_data(&user_input, &mut users_data) {
                eprintln!("Failed to add user: {}", e);
            } else {
                save_users_data(&users_data);
                println!("User added: {}", email);
            }
        }
        "login" => {
            if args.len() != 4 {
                eprintln!("Usage: {} login <email> <password>", args[0]);
                return;
            }
            let email = &args[2];
            let password = &args[3];
            let users_data = load_users_data();
            let user_input = UserInput::new(email.clone(), password.clone());
            match log_to_vaultify(&user_input, &users_data) {
                Ok(jwt) => println!("Login successful: {}", jwt.get_email()),
                Err(e) => eprintln!("Login failed: {}", e),
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Use one of the following commands: init-config, init-vault, load-vault, add-user, login");
        }
    }
}
