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
}

/**
 * This function init a vault at a
 * specific path and assign the given user
 * as administrator
 */

pub fn init_vault(user: &LocalJWT, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match { create_dir(&path) } {
        Err(_) => return Err(Box::new(ErrorType::ArgumentError)),
        _ => {}
    }

    init_config_vault(&path);


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
 * This function is uses to init the config directory of a vault
 * (.vault)
 */

fn init_config_vault(path: &str) {
    let mut path_config_root = path.to_string().clone();
    path_config_root.push_str(VAULT_CONFIG_ROOT);

    let mut path_users_dir = path_config_root.clone();
    path_users_dir.push_str(VAULT_USERS_DIR);

    let mut path_users_data = path_users_dir.clone();
    path_users_data.push_str(USERS_DATA);

    create_dir(&path_config_root).expect("Could not create folder");
    create_dir(&path_users_dir).expect("Could not create folder");
    File::create(&path_users_data).expect("Could not create files");
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
    let mut path = VAULTIFY_CONFIG.to_string();
    path.push_str(USERS_DATA);
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
