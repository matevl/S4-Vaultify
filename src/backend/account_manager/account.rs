use crate::backend::USERS_DATA;
use crate::error_manager::ErrorType;
use bcrypt::{hash, verify};
use rand::Rng;
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Serialize, Serializer};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use s4_vaultify::error_manager::ErrorType;

#[derive(Debug)]
pub enum Perms {
    Admin,
    Write,
    Read,
}

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
 * All the nada that related to a specific User
 * If it matches with a user, it will be encapsulated by a JWT
 */

pub struct UserData {
    hash_email: String,
    hash_pw: String,
    perms: Perms,
}

impl UserData {
    pub fn new(email: &String, password: &String, perms: Perms) -> UserData {
        let cost_email = rand::rng().random_range(4..=31);
        let cost_pw = rand::rng().random_range(4..=31);
        UserData {
            hash_email: hash(&email.clone(), cost_email).expect("Failed to hash password"),
            hash_pw: hash(&password.clone(), cost_pw).expect("Failed to hash password"),
            perms,
        }
    }
    pub fn get_hash_email(&self) -> &str {
        &self.hash_email
    }
    pub fn get_hash_pw(&self) -> &str {
        &self.hash_pw
    }
    pub fn get_permissions(&self) -> &Perms {
        &self.perms
    }
}

impl Clone for UserData {
    fn clone(&self) -> UserData {
        let perms = match &self.perms {
            Perms::Admin => Perms::Admin,
            Perms::Write => Perms::Write,
            Perms::Read => Perms::Read,
        };
        UserData::new(&self.hash_email, &self.hash_pw, perms)
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

        let perms = map
            .get("perms")
            .ok_or_else(|| de::Error::custom("Missing permissions"))
            .and_then(|v| serde_json::from_value(v.clone()).map_err(de::Error::custom))?;

        Ok(UserData {
            hash_email,
            hash_pw,
            perms,
        })
    }
}

impl Serialize for UserData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("UserData", 3)?;
        state.serialize_field("hash_email", &self.hash_email)?;
        state.serialize_field("hash_pw", &self.hash_pw)?;
        state.serialize_field("perms", &self.perms)?;

        state.end()
    }
}

/**
 * Encapsulation of UserData for logged Users
 * In case of error the validity of this token could be remove
 */

pub struct JWT {
    email: String,
    user_data: UserData,
    exp: usize,
}

impl JWT {
    pub fn new(user_data: UserData, email: String) -> JWT {
        JWT {
            email,
            user_data,
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
    pub fn get_data(&self) -> &UserData {
        &self.user_data
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

pub fn local_log_in(
    user: &UserInput,
    users_data: Vec<UserData>,
) -> Result<JWT, Box<dyn std::error::Error>> {
    for data in users_data {
        match verify(&data.hash_email, &user.email) {
            Ok(true) => match verify(&data.hash_pw, &user.password) {
                Ok(true) => {
                    return Ok(JWT::new(data.clone(), user.email.clone()));
                }
                _ => continue,
            },
            _ => continue,
        }
    }

    Err(Box::new(ErrorType::LoginError))
}

pub fn load_users_data(path: &str) -> Vec<UserData> {
    let mut path = path.to_string();
    path.push_str(USERS_DATA);

    let mut file = std::fs::File::open(path).expect("Unable to open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let users_data = serde_json::from_str(&contents).expect("Unable to parse JSON");

    users_data
}

pub fn sava_users_data(users_data: &Vec<UserData>, path: &str) {
    let mut path = path.to_string();
    path.push_str(USERS_DATA);

    let content = serde_json::to_string(users_data).expect("Unable to serialize user data");
    let mut file = std::fs::File::open(path).expect("Unable to open file");

    file.write_all(&content.as_bytes())
        .expect("Unable to write file");
}
