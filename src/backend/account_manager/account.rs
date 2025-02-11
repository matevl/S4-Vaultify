use crate::error_manager::ErrorType;
use bcrypt::{hash, DEFAULT_COST};
use rand::Rng;
use serde_json::{Deserializer, Serializer};
use std::io::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum Permissions {
    Reading = 0,
    Writing = 1,
    Admin = 2,
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

    pub fn get_hash_password(&self, sel: u32) -> String {
        let cost = if sel >= 4 && sel <= 31 {
            sel
        } else {
            DEFAULT_COST
        };
        hash(&self.password, cost).expect("Failed to hash password")
    }
}

/**
 * All the nada that related to a specific User
 * If it matches with a user, it will be encapsulated by a JWT
 */

struct UserData {
    hash_email: String,
    hash_pw: String,
    salt_email: u32,
    salt_pw: u32,
    permissions: Permissions,
}

impl UserData {
    fn new(email: String, password: String, permissions: Permissions) -> UserData {
        let salt_email = rand::thread_rng().gen_range(4..=31);
        let salt_pw = rand::thread_rng().gen_range(4..=31);
        UserData {
            hash_email: hash(&email, salt_email).expect("Failed to hash password"),
            hash_pw: hash(&password, salt_pw).expect("Failed to hash password"),
            salt_email,
            salt_pw,
            permissions,
        }
    }
    fn get_hash_email(&self) -> &str {
        &self.hash_email
    }
    fn get_hash_pw(&self) -> &str {
        &self.hash_pw
    }
    fn get_salt_email(&self) -> u32 {
        self.salt_email
    }
    fn get_salt_pw(&self) -> u32 {
        self.salt_pw
    }
    fn get_permissions(&self) -> &Permissions {
        &self.permissions
    }
}

/**
 * Encapsulation of UserData for logged Users
 * In case of error the validity of this token could be remove
 */

struct JWT {
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
    pub fn get_permissions(&self) -> &Permissions {
        &self.user_data.permissions
    }

    pub fn get_hash_pw(&self) -> &str {
        &self.user_data.hash_pw
    }

    pub fn get_salt_email(&self) -> u32 {
        self.user_data.get_salt_email()
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
