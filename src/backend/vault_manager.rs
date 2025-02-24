use crate::backend::account_manager::account::Perms::Admin;
use crate::backend::aes_keys::crypted_key::encrypt;
use crate::backend::aes_keys::keys_password::generate_random_key;
use crate::backend::{account_manager, *};
use crate::error_manager::VaultError;
use account_manager::account::*;
use actix_web::dev::ResourcePath;
use dirs;
use std::collections::HashMap;
use std::fs::{create_dir, exists, File};
use std::fs;

/**
 * Struct representing the environment of the current vault.
 * - `local_users`: All users in the vault.
 * - `loaded_vault_path`: The path of the vault.
 * - `logged_users`: The currently logged-in account.
 */
pub struct VaultManager {
    pub local_users: Vec<UserData>,
    pub loaded_vault_path: String,
    pub logged_users: Option<JWT>,
}

impl VaultManager {
    /**
     * Create a new VaultManager instance.
     */
    pub fn new(local_users: Vec<UserData>, loaded_vault_path: &str) -> VaultManager {
        VaultManager {
            local_users,
            loaded_vault_path: loaded_vault_path.to_string(),
            logged_users: None,
        }
    }

    /**
     * Initialize the configuration directories for the vault.
     */
    fn init_config_vault(&self) {
        create_dir(&format!("{}{}", &self.loaded_vault_path, VAULT_CONFIG_ROOT))
            .expect("Could not create folder");
        create_dir(&format!("{}{}", &self.loaded_vault_path, VAULT_USERS_DIR))
            .expect("Could not create folder");
        let key = generate_random_key();
        let vault_jwt = VaultJWT::new(Admin, key.as_slice());
        let content = serde_json::to_string(&vault_jwt).unwrap();
        let encrypted_content = encrypt(content.as_bytes(), key.as_slice());
        File::create(&format!(
            "{}{}{}",
            &self.loaded_vault_path,
            VAULT_USERS_DIR,
            &self
                .logged_users
                .as_ref()
                .unwrap()
                .get_user_data()
                .get_hash_email()
        ))
        .expect("TODO: panic message");
    }

    /**
     * Load a vault using the logged-in user in Vaultify and provide the necessary environment to work.
     */
    pub fn load_vault(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Implement the logic to load the vault using the logged-in user
        get_access_to_vault(self.logged_users.as_mut().unwrap(), path)
    }
}

/**
 * Initialize a vault at a specific path and assign the given user as administrator.
 */
pub fn init_vault(
    vault_manager: &mut VaultManager,
    new_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    vault_manager.loaded_vault_path = new_path.to_string();

    create_dir(&vault_manager.loaded_vault_path)?;

    vault_manager.init_config_vault();
    Ok(())
}

/**
 * Initialize the main configuration for the software to ensure it exists.
 */
pub fn init_config_vaultify() {
    let root = dirs::home_dir().expect("Could not find home dir");

    let vaultify_config_path = root.join(VAULTIFY_CONFIG);
    if !vaultify_config_path.exists() {
        fs::create_dir_all(&vaultify_config_path).expect("Could not create folder");
    }

    let users_data_path = vaultify_config_path.join(USERS_DATA);
    if !users_data_path.exists() {
        fs::create_dir_all(users_data_path.parent().unwrap()).expect("Could not create directory");
        File::create(&users_data_path).expect("Could not create file");
        let empty: Vec<UserData> = Vec::new();
        save_users_data(&empty);
    }

    let vault_matching_path = vaultify_config_path.join(VAULT_MATCHING);
    if !vault_matching_path.exists() {
        fs::create_dir_all(vault_matching_path.parent().unwrap()).expect("Could not create directory");
        File::create(&vault_matching_path).expect("Could not create file");
        let empty: HashMap<String, Vec<(String, String)>> = HashMap::new();
        save_vault_matching(&empty);
    }
}

