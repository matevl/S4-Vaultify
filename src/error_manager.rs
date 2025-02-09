
#[derive(PartialEq)]
#[allow(dead_code)]
pub enum ErrorType {
    ArgumentError,
    TypeFilesError,
    ErrorTypeError, // If the error cached is not the one excepted
}

impl core::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorType::ArgumentError => write!(f, "ArgumentError"),
            ErrorType::TypeFilesError => write!(f, "TypeFilesError"),
            ErrorType::ErrorTypeError => write!(f, "ErrorTypeError"),
        }
    }
}

impl core::fmt::Debug for ErrorType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}
