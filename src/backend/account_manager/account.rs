use crate::backend::USERS_DATA;
use crate::error_manager::ErrorType;
use bcrypt::hash;
use rand::Rng;
use serde::{de, Deserialize, Serialize};
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
enum Perms {
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
        S: serde::Serializer,
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
    email: String,
    password: String,
}

impl UserInput {
    pub fn new(email: String, password: String) -> UserInput {
        UserInput { email, password }
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }
}

/**
 * All the nada that related to a specific User
 * If it matches with a user, it will be encapsulated by a JWT
 */

struct UserData {
    hash_email: String,
    hash_pw: String,
    perms: Perms,
}

impl UserData {
    fn new(email: String, password: String, perms: Perms) -> UserData {
        let cost_email = rand::rng().random_range(4..=31);
        let cost_pw = rand::rng().random_range(4..=31);
        UserData {
            hash_email: hash(&email, cost_email).expect("Failed to hash password"),
            hash_pw: hash(&password, cost_pw).expect("Failed to hash password"),
            perms,
        }
    }
    fn get_hash_email(&self) -> &str {
        &self.hash_email
    }
    fn get_hash_pw(&self) -> &str {
        &self.hash_pw
    }
    fn get_permissions(&self) -> &Perms {
        &self.perms
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
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(de::Error::custom)
            })?;

        Ok(UserData {
            hash_email,
            hash_pw,
            perms,
        })
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
    pub fn get_permissions(&self) -> &Perms {
        &self.user_data.perms
    }

    pub fn get_hash_pw(&self) -> &str {
        &self.user_data.hash_pw
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

pub fn local_log_in(user: &UserInput) -> Result<JWT, Box<dyn std::error::Error>> {
    Err(Box::new(ErrorType::LoginError))
}

pub fn load_users_data() -> Vec<UserData> {
    let mut file = std::fs::File::open(USERS_DATA).expect("Unable to open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    //serde_json::from_str(&contents).expect("Unable to parse JSON")

    vec![]
}
