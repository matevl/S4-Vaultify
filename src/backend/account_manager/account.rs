use crate::error_manager::ErrorType;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum Permissions {
    Reading = 0,
    Writing = 1,
    Admin = 2,
}

pub struct User {
    email: String,
    password: String,
}

impl User {
    pub fn new(email: String, password: String) -> User {
        User { email, password }
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }
}

struct JWT {
    id: i32,
    permissions: Permissions,
    exp: usize,
}

impl JWT {
    pub fn new(id: i32, permissions: Permissions) -> JWT {
        JWT {
            id,
            permissions,
            exp: {
                (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 3600) as usize
            },
        }
    }
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_permissions(&self) -> &Permissions {
        &self.permissions
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

pub fn local_log_in(user: &User) -> Result<JWT, Err()> {
    Err(ErrorType::LoginError)
}