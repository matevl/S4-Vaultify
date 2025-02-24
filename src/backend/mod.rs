use crate::backend::account_manager::account::{UserData, JWT};

pub mod account_manager;
pub mod encryption;
pub mod file_manager;
pub mod vault_manager;

pub mod aes_keys;

// Backend Const
const VAULT_CONFIG_ROOT: &str = ".vault/";
const VAULT_USERS_DIR: &str = ".vault/users/";
const USERS_DATA: &str = "users_data.json";
pub const VAULTIFY_CONFIG: &str = ".vaultify/";

const VAULT_MATCHING: &str = ".vaultify/vaultmatching.json";
