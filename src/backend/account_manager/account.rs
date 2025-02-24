use crate::backend::aes_keys::decrypted_key::decrypt;
use crate::backend::aes_keys::keys_password::{derive_key, generate_salt_from_login};
use crate::backend::{USERS_DATA, VAULTIFY_CONFIG, VAULT_MATCHING, VAULT_USERS_DIR};
use crate::error_manager::VaultError;
use bcrypt::{hash, verify};
use dirs;
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fs;
use std::fs::{exists, File};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

/**
 * Enum to handle permission verification.
 * Each variant represents a different level of access.
 */
#[derive(Debug)]
pub enum Perms {
    Admin,
    Write,
    Read,
}

impl Perms {
    /**
     * Check if the permission allows reading.
     */
    fn can_read(&self) -> bool {
        matches!(self, Perms::Admin | Perms::Write | Perms::Read)
    }

    /**
     * Check if the permission allows writing.
     */
    fn can_write(&self) -> bool {
        matches!(self, Perms::Admin | Perms::Write)
    }

    /**
     * Check if the permission allows execution (admin-only).
     */
    fn can_execute(&self) -> bool {
        matches!(self, Perms::Admin)
    }
}

impl Clone for Perms {
    fn clone(&self) -> Self {
        match self {
            Perms::Admin => Perms::Admin,
            Perms::Write => Perms::Write,
            Perms::Read => Perms::Read,
        }
    }
}

// JSON Manipulation for Perms
impl<'de> Deserialize<'de> for Perms {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val: u8 = Deserialize::deserialize(deserializer)?;
        match val {
            4 => Ok(Perms::Admin),
            2 => Ok(Perms::Write),
            1 => Ok(Perms::Read),
            _ => Err(de::Error::custom("Invalid permission value")),
        }
    }
}

impl Serialize for Perms {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value: u8 = match self {
            Perms::Admin => 4,
            Perms::Write => 2,
            Perms::Read => 1,
        };
        serializer.serialize_u8(value)
    }
}

/**
 * Struct representing user input from the GUI.
 */
#[derive(Debug)]
pub struct UserInput {
    pub email: String,
    pub password: String,
}

impl UserInput {
    /**
     * Create a new UserInput instance.
     */
    pub fn new(email: String, password: String) -> UserInput {
        UserInput { email, password }
    }
}

/**
 * Struct representing user data.
 * Contains hashed email and password for secure storage.
 */
pub struct UserData {
    hash_email: String,
    hash_pw: String,
}

impl UserData {
    /**
     * Create a new LocalUserData instance with hashed email and password.
     */
    pub fn new(email: &str, password: &str) -> UserData {
        let cost = 12; // Use a fixed cost factor for bcrypt
        UserData {
            hash_email: hash(email, cost).expect("Failed to hash email"),
            hash_pw: hash(password, cost).expect("Failed to hash password"),
        }
    }

    /**
     * Get the hashed email.
     */
    pub fn get_hash_email(&self) -> &str {
        &self.hash_email
    }

    /**
     * Get the hashed password.
     */
    pub fn get_hash_pw(&self) -> &str {
        &self.hash_pw
    }
}

impl Clone for UserData {
    /**
     * Clone the LocalUserData instance.
     */
    fn clone(&self) -> UserData {
        UserData::new(&self.hash_email, &self.hash_pw)
    }
}

impl<'de> Deserialize<'de> for UserData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = serde_json::Value::deserialize(deserializer)?;
        let hash_email = map
            .get("hash_email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| de::Error::custom("Missing or invalid hash_email"))?
            .to_string();

        let hash_pw = map
            .get("hash_pw")
            .and_then(|v| v.as_str())
            .ok_or_else(|| de::Error::custom("Missing or invalid hash_pw"))?
            .to_string();

        Ok(UserData {
            hash_email,
            hash_pw,
        })
    }
}

impl Serialize for UserData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("UserData", 2)?;
        state.serialize_field("hash_email", &self.hash_email)?;
        state.serialize_field("hash_pw", &self.hash_pw)?;
        state.end()
    }
}

/**
 * Struct representing a vault JWT with permissions and vault key.
 * - `perms`: Permissions associated with the JWT.
 * - `vault_key`: The key used to encrypt and decrypt vault data.
 */
pub struct VaultJWT {
    perms: Perms,
    vault_key: Box<[u8]>,
}

impl VaultJWT {
    /**
     * Create a new VaultJWT instance.
     *
     * @param perms - Permissions for the JWT.
     * @param vault_key - The key for vault encryption/decryption.
     */
    pub fn new(perms: Perms, vault_key: &[u8]) -> VaultJWT {
        VaultJWT {
            perms,
            vault_key: Box::from(vault_key),
        }
    }
}

impl Clone for VaultJWT {
    /**
     * Clone the VaultJWT instance.
     */
    fn clone(&self) -> VaultJWT {
        VaultJWT::new(self.perms.clone(), self.vault_key.as_ref())
    }
}

impl<'de> Deserialize<'de> for VaultJWT {
    /**
     * Deserialize a VaultJWT instance from a data format.
     */
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct VaultJWTVisitor {
            perms: Perms,
            vault_key: Vec<u8>,
        }

        let visitor = VaultJWTVisitor::deserialize(deserializer)?;
        Ok(VaultJWT::new(visitor.perms, &visitor.vault_key))
    }
}

impl Serialize for VaultJWT {
    /**
     * Serialize a VaultJWT instance into a data format.
     */
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("VaultJWT", 2)?;
        state.serialize_field("perms", &self.perms)?;
        state.serialize_field("vault_key", &self.vault_key)?;
        state.end()
    }
}

/**
 * Struct representing a local JWT for logged-in users.
 */

pub struct JWT {
    email: String,
    user_data: UserData,
    user_key: Box<[u8]>,
    vault_access: Option<VaultJWT>,
    exp: usize,
}
impl Serialize for JWT {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("JWT", 5)?;
        state.serialize_field("email", &self.email)?;
        state.serialize_field("user_data", &self.user_data)?;
        state.serialize_field("user_key", &self.user_key)?;
        state.serialize_field("vault_access", &self.vault_access)?;
        state.serialize_field("exp", &self.exp)?;
        state.end()
    }
}

// Désérialisation de JWT
impl<'de> Deserialize<'de> for JWT {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct JWTVisitor {
            email: String,
            user_data: UserData,
            user_key: Vec<u8>,
            vault_access: Option<VaultJWT>,
            exp: usize,
        }

        let visitor = JWTVisitor::deserialize(deserializer)?;
        Ok(JWT {
            email: visitor.email,
            user_data: visitor.user_data,
            user_key: Box::from(visitor.user_key),
            vault_access: visitor.vault_access,
            exp: visitor.exp,
        })
    }
}

impl JWT {
    /**
     * Create a new LocalJWT instance.
     */
    pub fn new(user_data: UserData, email: String, user_key: &[u8]) -> JWT {
        JWT {
            email,
            user_data,
            user_key: Box::from(user_key),
            vault_access: None,
            exp: (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 3600) as usize,
        }
    }

    /**
     * Get the user's email.
     */
    pub fn get_email(&self) -> &str {
        &self.email
    }

    /**
     * Get the user's data.
     */
    pub fn get_user_data(&self) -> &UserData {
        &self.user_data
    }

    /**
     * Get the vault access.
     */
    pub fn get_vault_access(&self) -> &Option<VaultJWT> {
        &self.vault_access
    }

    /**
     * Get the expiration time.
     */
    pub fn get_exp(&self) -> usize {
        self.exp
    }

    /**
     * Check if the JWT is valid.
     */
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        now < self.exp
    }

    /**
     * Invalidate the JWT.
     */
    pub fn kill(&mut self) {
        self.exp = 0;
    }
}

/**
 * Log in to Vaultify and return a LocalJWT if successful.
 */
pub fn log_to_vaultify(
    user: &UserInput,
    users_data: &Vec<UserData>,
) -> Result<JWT, Box<dyn std::error::Error>> {
    for data in users_data {
        if verify(user.email.as_str(), data.get_hash_email())?
            && verify(user.password.as_str(), data.get_hash_pw())?
        {
            let salt = generate_salt_from_login(user.email.as_str());
            let key = derive_key(user.password.as_str(), salt.as_slice(), 100000); // Increase iterations for security
            return Ok(JWT::new(data.clone(), user.email.clone(), key.as_slice()));
        }
    }

    Err(Box::new(VaultError::LoginError))
}

/**
 * Grant access to the vault if the user is in the vault database.
 */
pub fn get_access_to_vault(jwt: &mut JWT, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let user_path = format!(
        "{}{}{}",
        path,
        VAULT_USERS_DIR,
        jwt.get_user_data().get_hash_email()
    );

    if exists(&user_path)? {
        pub fn get_access_to_vault(
            jwt: &mut JWT,
            path: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let user_path = format!(
                "{}{}{}",
                path,
                VAULT_USERS_DIR,
                jwt.get_user_data().get_hash_email()
            );

            if exists(&user_path)? {
                let crypted_content = std::fs::read_to_string(&user_path)?;
                let decypted_content: String = decrypt(crypted_content.as_bytes(), &jwt.user_key)?
                    .iter()
                    .map(|&c| c as char)
                    .collect();
                jwt.vault_access = Some(serde_json::from_str(&decypted_content)?);
                Ok(())
            } else {
                Err(Box::new(VaultError::LoginError))
            }
        }
        let crypted_content = std::fs::read_to_string(&user_path)?;

        let decrypted_byte = decrypt(crypted_content.as_bytes(), &jwt.user_key)?;
        let decrypted_content: String = String::from_utf8(decrypted_byte)?;

        let vault_access: VaultJWT = serde_json::from_str(&decrypted_content)?;
        jwt.vault_access = Some(vault_access);

        Ok(())
    } else {
        Err(Box::new(VaultError::LoginError))
    }
}

/**
 * Load user data from a file.
 */
pub fn load_users_data() -> Vec<UserData> {
    let root = dirs::home_dir().expect("No home dir");
    let file_path = root.join(format!("{}{}", VAULTIFY_CONFIG, USERS_DATA));

    let contents = fs::read_to_string(file_path).expect("Unable to read file");
    serde_json::from_str(&contents).expect("Unable to parse JSON")
}

/**
 * Save user data to a file.
 */
pub fn save_users_data(users_data: &Vec<UserData>) {
    let root = dirs::home_dir().expect("No home dir");
    let file_path = root.join(format!("{}{}", VAULTIFY_CONFIG, USERS_DATA));

    let content = serde_json::to_string(users_data).expect("Unable to serialize user data");
    fs::write(file_path, content.as_bytes()).unwrap()
}

/**
 * Add a new user to the user data.
 */
pub fn add_user_to_data(
    user_input: &UserInput,
    users_data: &mut Vec<UserData>,
) -> Result<(), Box<dyn std::error::Error>> {
    for data in users_data.iter() {
        if verify(&user_input.email, &data.hash_email)? {
            return Err(Box::new(VaultError::ArgumentError));
        }
    }
    users_data.push(UserData::new(&user_input.email, &user_input.password));
    Ok(())
}

type VaultMatching = HashMap<String, Vec<(String, String)>>;

pub fn load_vault_matching() -> VaultMatching {
    let root = dirs::home_dir().expect("No home dir");
    let file_path = root.join(format!("{}{}", VAULTIFY_CONFIG, VAULT_MATCHING));

    let file_content = fs::read_to_string(file_path).expect("Unable to read file");
    let vault_matching: VaultMatching =
        serde_json::from_str(&file_content).expect("Unable to parse JSON");
    vault_matching
}

// Function to save vault matching to a JSON file
pub fn save_vault_matching(data: &VaultMatching) {
    let root = dirs::home_dir().expect("No home dir");
    let file_path = root.join(format!("{}{}", VAULTIFY_CONFIG, VAULT_MATCHING));

    let file_content = serde_json::to_string_pretty(data).expect("Unable to serialize user data");
    fs::write(file_path, file_content).expect("Unable to write file");
}
