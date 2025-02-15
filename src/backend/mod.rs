pub mod account_manager;
pub mod encryption;
pub mod file_manager;
pub mod vault_manager;

pub mod aes_keys;

// Backend Const
const CONFIG_ROOT: &str = ".vault/";
const USERS_DIR: &str = ".vault/users/";
const USERS_DATA: &str = ".vault/users/data.json";
