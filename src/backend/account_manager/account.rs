use crate::backend::aes_keys::keys_password::{derive_key, generate_salt_from_login};
use crate::backend::{USERS_DATA, VAULT_USERS_DIR};
use crate::error_manager::ErrorType;
use bcrypt::{hash, verify};
use rand::Rng;
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Serialize, Serializer};
use std::fs::{exists, Permissions};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

/**
 * This Enum is useful to handle
 * permission verification
 */
#[derive(Debug)]
pub enum Perms {
    Admin,
    Write,
    Read,
}
#[allow(dead_code)]
impl Perms {
    fn can_read(&self) -> bool {
        matches!(self, Perms::Admin | Perms::Write | Perms::Read)
    }

    fn can_write(&self) -> bool {
        matches!(self, Perms::Admin | Perms::Write)
    }

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
        type Value = u8;
        let val = Value::deserialize(deserializer)?;
        match val {
            4 => Ok(Perms::Admin),
            2 => Ok(Perms::Write),
            1 => Ok(Perms::Read),
            _ => Err(de::Error::custom("Invalid perms")),
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
 * Just an abstraction of input give on the GUI
 */
pub struct UserInput {
    pub email: String,
    pub password: String,
}

impl UserInput {
    pub fn new(email: String, password: String) -> UserInput {
        UserInput { email, password }
    }
}

/**
 * All the dada that are link to a specific User
 * If it matches with a user, it will be encapsulated by a JWT
 */

pub struct LocalUserData {
    hash_email: String,
    hash_pw: String,
}

impl LocalUserData {
    pub fn new(email: &str, password: &str) -> LocalUserData {
        let cost_email = rand::rng().random_range(4..=31);
        let cost_pw = rand::rng().random_range(4..=31);
        LocalUserData {
            hash_email: hash(&email.to_string(), cost_email).expect("Failed to hash password"),
            hash_pw: hash(&password.to_string(), cost_pw).expect("Failed to hash password"),
        }
    }
    pub fn get_hash_email(&self) -> &str {
        &self.hash_email
    }
    pub fn get_hash_pw(&self) -> &str {
        &self.hash_pw
    }
}

impl Clone for LocalUserData {
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
        let mut state = serializer.serialize_struct("UserData", 3)?;
        state.serialize_field("hash_email", &self.hash_email)?;
        state.serialize_field("hash_pw", &self.hash_pw)?;

        state.end()
    }
}

pub struct VaultJWT {
    perms: Perms,
    vault_key: Box<[u8]>,
}

impl VaultJWT {
    pub fn new(perms: Perms, vault_key: &[u8]) -> VaultJWT {
        VaultJWT {
            perms,
            vault_key: Box::from(vault_key),
        }
    }
}

/**
 * Encapsulation of UserData for logged Users
 * In case of error the validity of this token could be removed
 */

pub struct LocalJWT {
    email: String,
    user_data: LocalUserData,
    user_key: Box<[u8]>,
    vault_access: Option<VaultJWT>,
    exp: usize,
}
#[allow(dead_code)]
impl LocalJWT {
    pub fn new(user_data: LocalUserData, email: String, user_key: &[u8]) -> LocalJWT {
        LocalJWT {
            email,
            user_data,
            user_key: Box::from(user_key),
            vault_access: None,
            exp: {
                (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 3600) as usize
            },
        }
    }
    pub fn get_email(&self) -> &str {
        &self.email
    }
    pub fn get_local_users_data(&self) -> &LocalUserData {
        &self.user_data
    }

    pub fn get_vault_access(&self) -> &Option<VaultJWT> {
        &self.vault_access
    }

    pub fn get_exp(&self) -> usize {
        self.exp
    }

    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        now > self.exp
    }

    pub fn kill(&mut self) {
        self.exp = 0;
    }
}

/**
 * Give the local JWT so it will be used for
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
            let key = derive_key(user.password.as_str(), salt.as_slice(), 100);
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
 * This function use the account Logged and
 * generate and JWT if it is in the vault database
 */

pub fn get_access_to_vault(
    local_jwt: &mut LocalJWT,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match exists(format!(
        "{}{}{}",
        path,
        VAULT_USERS_DIR,
        local_jwt.get_local_users_data().hash_email
    )) {
        Ok(true) => {
            // Function do decrupt..
            let vault_key: [u8; 1] = [1];
            let perms = Perms::Admin;
            //
            local_jwt.vault_access = Some(VaultJWT::new(perms, vault_key.as_ref()));
            Ok(())
        }
        _ => Err(Box::new(ErrorType::LoginError)),
    }
}

/**
 * Could load data from local users or user in a vault
 */
pub fn load_users_data(path: &str) -> Vec<LocalUserData> {
    let mut path = path.to_string();
    path.push_str(USERS_DATA);

    let mut file = std::fs::File::open(path).expect("Unable to open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let users_data = serde_json::from_str(&contents).expect("Unable to parse JSON");

    users_data
}

/**
 * Could save data from local users or user in a vault
 */
pub fn sava_users_data(users_data: &Vec<LocalUserData>, path: &str) {
    let mut path = path.to_string();
    path.push_str(USERS_DATA);

    let content = serde_json::to_string(users_data).expect("Unable to serialize user data");
    let mut file = std::fs::File::open(path).expect("Unable to open file");

    file.write_all(&content.as_bytes())
        .expect("Unable to write file");
}

fn add_user_to_data(
    user_input: UserInput,
    users_data: &mut Vec<LocalUserData>,
    perms: Perms,
) -> Result<(), Box<dyn std::error::Error>> {
    for data in users_data.iter() {
        match verify(&user_input.email, &data.hash_email) {
            Ok(true) => {
                return Err(Box::new(ErrorType::ArgumentError));
            }
            _ => {}
        }
    }
    users_data.push(LocalUserData::new(&user_input.email, &user_input.password));

    Ok(())
}
