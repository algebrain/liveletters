use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecretBoxError {
    #[error("io error: {message}")]
    Io {
        #[source]
        source: std::io::Error,
        message: String,
    },

    #[error("invalid key length at {path}: expected {expected} bytes, got {actual}")]
    InvalidKeyLength {
        path: PathBuf,
        expected: usize,
        actual: usize,
    },

    #[error("invalid protected secret format: {message}")]
    InvalidFormat { message: String },

    #[error("crypto error: {message}")]
    Crypto { message: String },
}

impl SecretBoxError {
    pub fn io(error: std::io::Error, message: impl Into<String>) -> Self {
        Self::Io {
            source: error,
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for SecretBoxError {
    fn from(value: std::io::Error) -> Self {
        let message = value.to_string();
        Self::Io {
            source: value,
            message,
        }
    }
}
