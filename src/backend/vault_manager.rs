use crate::backend::{account_manager, *};
use crate::error_manager::ErrorType;
use account_manager::account::*;
use std::fs::{create_dir, exists, File};

/**
 * This function init a vault at a
 * specific path and assign the given user
 * as administrator
 */

pub fn init_vault(user: &JWT, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match { create_dir(&path) } {
        Err(_) => return Err(Box::new(ErrorType::ArgumentError)),
        _ => {}
    }

    init_config_vault(&path);

    let users_data = vec![UserData::new(
        &user.get_data().get_hash_email().to_string(),
        &user.get_data().get_hash_pw().to_string(),
        Perms::Admin,
    )];
    sava_users_data(&users_data, path);
    Ok(())
}

/**
 * This function load a vault using
 * the users logged in Vaultify,
 * and give the environment needed
 * to work.
 */
pub fn load_vault(user: &JWT, path: &str) -> Result<VaultEnv, Box<dyn std::error::Error>> {
    match exists(path) {
        Ok(true) => {
            let mut env = VaultEnv::new(load_users_data(path), path);
            match log_to_vault(user, &env.users_data) {
                Ok(logged_users) => {
                    env.logged_users = Some(logged_users);
                    Ok(env)
                }
                _ => Err(Box::new(ErrorType::LoginError)),
            }
        }
        _ => Err(Box::new(ErrorType::ArgumentError)),
    }
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
