use crate::backend::account_manager::account::{UserData, JWT};

pub mod account_manager;
pub mod encryption;
pub mod file_manager;
pub mod vault_manager;

// Backend Const
const VAULT_CONFIG_ROOT: &str = ".vault/";
const VAULT_USERS_DIR: &str = ".vault/users/";
const USERS_DATA: &str = "users_data.json";
const VAULTIFY_CONFIG: &str = "~/.vaultify/";

pub struct VaultEnv {
    pub users_data: Vec<UserData>,
    pub vault_path: String,
    pub logged_users: Option<JWT>,
}

impl VaultEnv {
    pub fn new(users_data: Vec<UserData>, vault_path: &str) -> VaultEnv {
        VaultEnv {
            users_data,
            vault_path: vault_path.to_string(),
            logged_users: None,
        }
    }
}
