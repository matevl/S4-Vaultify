use crate::backend::aes_keys::keys_password::{derive_key, generate_salt_from_login};
use crate::backend::{USERS_DATA, VAULT_USERS_DIR};
use crate::error_manager::ErrorType;
use bcrypt::{hash, verify};
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Serialize, Serializer};
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
pub struct LocalUserData {
    hash_email: String,
    hash_pw: String,
}

impl LocalUserData {
    /**
     * Create a new LocalUserData instance with hashed email and password.
     */
    pub fn new(email: &str, password: &str) -> LocalUserData {
        let cost = 12; // Use a fixed cost factor for bcrypt
        LocalUserData {
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

impl Clone for LocalUserData {
    /**
     * Clone the LocalUserData instance.
     */
    fn clone(&self) -> LocalUserData {
        LocalUserData::new(&self.hash_email, &self.hash_pw)
    }
}

impl<'de> Deserialize<'de> for LocalUserData {
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

        Ok(LocalUserData {
            hash_email,
            hash_pw,
        })
    }
}

impl Serialize for LocalUserData {
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
 */
pub struct VaultJWT {
    perms: Perms,
    vault_key: Box<[u8]>,
}

impl VaultJWT {
    /**
     * Create a new VaultJWT instance.
     */
    pub fn new(perms: Perms, vault_key: &[u8]) -> VaultJWT {
        VaultJWT {
            perms,
            vault_key: Box::from(vault_key),
        }
    }
}

/**
 * Struct representing a local JWT for logged-in users.
 */
pub struct LocalJWT {
    email: String,
    user_data: LocalUserData,
    user_key: Box<[u8]>,
    vault_access: Option<VaultJWT>,
    exp: usize,
}

impl LocalJWT {
    /**
     * Create a new LocalJWT instance.
     */
    pub fn new(user_data: LocalUserData, email: String, user_key: &[u8]) -> LocalJWT {
        LocalJWT {
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
    pub fn get_local_users_data(&self) -> &LocalUserData {
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
    users_data: Vec<LocalUserData>,
) -> Result<LocalJWT, Box<dyn std::error::Error>> {
    for data in users_data {
        if verify(user.email.as_str(), data.get_hash_email())?
            && verify(user.password.as_str(), data.get_hash_pw())?
        {
            let salt = generate_salt_from_login(user.email.as_str());
            let key = derive_key(user.password.as_str(), salt.as_slice(), 100000); // Increase iterations for security
            return Ok(LocalJWT::new(
                data.clone(),
                user.email.clone(),
                key.as_slice(),
            ));
        }
    }

    Err(Box::new(ErrorType::LoginError))
}

/**
 * Grant access to the vault if the user is in the vault database.
 */
pub fn get_access_to_vault(
    local_jwt: &mut LocalJWT,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let user_path = format!(
        "{}{}{}",
        path,
        VAULT_USERS_DIR,
        local_jwt.get_local_users_data().get_hash_email()
    );

    if exists(&user_path)? {
        // Decrypt the vault key and permissions here
        let vault_key: [u8; 32] = [0; 32]; // Replace with actual decryption logic
        let perms = Perms::Read; // Replace with actual permission logic

        local_jwt.vault_access = Some(VaultJWT::new(perms, &vault_key));
        Ok(())
    } else {
        Err(Box::new(ErrorType::LoginError))
    }
}

/**
 * Load user data from a file.
 */
pub fn load_users_data(path: &str) -> Vec<LocalUserData> {
    let file_path = format!("{}{}", path, USERS_DATA);
    let mut file = File::open(&file_path).expect("Unable to open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    serde_json::from_str(&contents).expect("Unable to parse JSON")
}

/**
 * Save user data to a file.
 */
pub fn save_users_data(users_data: &Vec<LocalUserData>, path: &str) {
    let file_path = format!("{}{}", path, USERS_DATA);
    let content = serde_json::to_string(users_data).expect("Unable to serialize user data");
    let mut file = File::create(&file_path).expect("Unable to create file");

    file.write_all(content.as_bytes())
        .expect("Unable to write file");
}

/**
 * Add a new user to the user data.
 */
fn add_user_to_data(
    user_input: UserInput,
    users_data: &mut Vec<LocalUserData>,
    perms: Perms,
) -> Result<(), Box<dyn std::error::Error>> {
    for data in users_data.iter() {
        if verify(&user_input.email, &data.hash_email)? {
            return Err(Box::new(ErrorType::ArgumentError));
        }
    }
    users_data.push(LocalUserData::new(&user_input.email, &user_input.password));
    Ok(())
}
