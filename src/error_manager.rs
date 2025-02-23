#[derive(PartialEq)]
#[allow(dead_code)]
pub enum VaultError {
    ArgumentError,
    TypeFilesError,
    LoginError,
    ErrorTypeError, // If the error cached is not the one excepted
}

impl core::fmt::Display for VaultError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VaultError::ArgumentError => write!(f, "ArgumentError"),
            VaultError::TypeFilesError => write!(f, "TypeFilesError"),
            VaultError::LoginError => write!(f, "LoginError"),
            VaultError::ErrorTypeError => write!(f, "ErrorTypeError"),
        }
    }
}

impl core::fmt::Debug for VaultError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for VaultError {}
