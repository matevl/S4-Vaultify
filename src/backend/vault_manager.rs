use crate::backend::{account_manager, *};
use crate::error_manager::ErrorType;
use account_manager::account::*;
use std::fs::{create_dir, exists, File};

/**
 * This struct contained the env of the current vault.
 * @users_data - All the users in the vault
 * @vault_path - The path of the vault
 * @logged_users - The current logged account
 */
pub struct VaultManager {
    pub local_users: Vec<LocalUserData>,
    pub loaded_vault_path: String,
    pub logged_users: Option<LocalJWT>,
}

impl VaultManager {
    pub fn new(local_users: Vec<LocalUserData>, loaded_vault_path: &str) -> VaultManager {
        VaultManager {
            local_users,
            loaded_vault_path: loaded_vault_path.to_string(),
            logged_users: None,
        }
    }
    fn init_config_vault(&self) {
        create_dir(&format!("{}{}", &self.loaded_vault_path, VAULT_CONFIG_ROOT))
            .expect("Could not create folder");
        create_dir(&format!("{}{}", &self.loaded_vault_path, VAULT_USERS_DIR))
            .expect("Could not create folder");
    }
}

/**
 * This function init a vault at a
 * specific path and assign the given user
 * as administrator
 */

pub fn init_vault(vault_manager: &mut VaultManager) -> Result<(), Box<dyn std::error::Error>> {
    match { create_dir(&vault_manager.loaded_vault_path) } {
        Err(_) => return Err(Box::new(ErrorType::ArgumentError)),
        _ => {}
    }

    vault_manager.init_config_vault();

    Ok(())
}

/**
 * This function load a vault using
 * the users logged in Vaultify,
 * and give the environment needed
 * to work.
 */
pub fn load_vault(user: &LocalJWT, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/**
 * MAIN CONFIG OF THE SOFTWARE
 * to be sure that it exist
 */
fn init_config_vaultify() {
    match exists(VAULTIFY_CONFIG) {
        Ok(true) => {}
        Ok(false) => create_dir(VAULTIFY_CONFIG).expect("Could not create folder"),
        Err(_) => {
            panic!("Could not find config file");
        }
    }
    let mut path = format!("{}{}", VAULTIFY_CONFIG, USERS_DATA);
    match exists(&path) {
        Ok(true) => {}
        Ok(false) => {
            File::create(&path).expect("Could not create folder");
        }
        Err(_) => {
            panic!("Could not find config file");
        }
    }
}
