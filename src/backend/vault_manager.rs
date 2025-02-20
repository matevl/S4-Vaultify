use crate::backend::{account_manager, *};
use crate::error_manager::ErrorType;
use account_manager::account::*;
use std::fs;
use std::fs::*;
pub fn init_vault(user_input: &UserInput, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match { create_dir(&path) } {
        Err(_) => return Err(Box::new(ErrorType::ArgumentError)),
        _ => {}
    }

    init_config(&path);

    let users_data = vec![UserData::new(
        &user_input.email,
        &user_input.password,
        Perms::Admin,
    )];
    sava_users_data(&users_data, path);
    Ok(())
}

fn init_config(path: &str) {
    let mut path_config_root = path.to_string().clone();
    path_config_root.push_str(CONFIG_ROOT);

    let mut path_users_dir = path_config_root.clone();
    path_users_dir.push_str(USERS_DIR);

    let mut path_users_data = path_users_dir.clone();
    path_users_data.push_str(USERS_DATA);

    create_dir(&path_config_root).expect("Could not create folder");
    create_dir(&path_users_dir).expect("Could not create folder");
    File::create(&path_users_data).expect("Could not create files");
}
