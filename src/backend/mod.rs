pub mod file_manager;
pub mod server_manager;

pub mod aes_keys;
pub mod auth;

pub mod file_flow;

// Backend Const
const VAULT_CONFIG_ROOT: &str = ".vault/";
const VAULT_USERS_DIR: &str = ".vault/users/";
pub const VAULTIFY_CONFIG: &str = ".vaultify/";
pub const USERS_DATA: &str = ".vaultify/users.json";
pub const VAULTS_MATCHING: &str = ".vaultify/vault_matching.json";
// Where the vault are stored
pub const VAULTS_DATA: &str = "VaultsData/";
pub const VAULTIFY_DATABASE: &str = ".vaultify/database.sqlite";

pub const PASSWORD: &str = "password.json";
