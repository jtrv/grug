use std::{error, fmt, io};

#[derive(Debug)]
pub enum AppError {
    IoError(io::Error),
    ParseIntError(std::num::ParseIntError),
    InvalidNumber(String),
    InvalidLineFormat(String),
    InvalidLineNumber(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO error: {}", e),
            AppError::ParseIntError(e) => write!(f, "Parse int error: {}", e),
            AppError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            AppError::InvalidLineFormat(s) => write!(f, "Invalid line format: {}", s),
            AppError::InvalidLineNumber(s) => write!(f, "Invalid line number: {}", s),
        }
    }
}

impl error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::IoError(e)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> Self {
        AppError::ParseIntError(e)
    }
}
