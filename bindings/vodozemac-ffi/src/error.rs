use std::string::FromUtf8Error;
use vodozemac::{DecodeError, KeyError};
use vodozemac::olm::{DecryptionError, SessionCreationError};

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum VodozemacError {
    #[error("Failed to acquire lock")]
    LockError,
    #[error("Unknown error")]
    Unknown,
    #[error("decode error: {error}")]
    ErrorDecode { error: String },
    #[error("decryption error: {error}")]
    ErrorDecryption { error: String },
    #[error("utf8 decode error: {error}")]
    ErrorUtf8Decode { error: String },
    #[error("key error: {error}")]
    KeyErrorDecode { error: String },
    #[error("session error: {error}")]
    SessionError { error: String },
    #[error("serde error: {error}")]
    SerdeError { error: String },
}

impl From<serde_json::Error> for VodozemacError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeError { error: err.to_string() }
    }
}

impl From<SessionCreationError> for VodozemacError {
    fn from(err: SessionCreationError) -> Self {
        Self::SessionError { error: err.to_string() }
    }
}

impl From<KeyError> for VodozemacError {
    fn from(err: KeyError) -> Self {
        Self::KeyErrorDecode { error: err.to_string() }
    }
}

impl From<DecodeError> for VodozemacError {
    fn from(err: DecodeError) -> Self {
        Self::ErrorDecode { error: err.to_string() }
    }
}

impl From<FromUtf8Error> for VodozemacError {
    fn from(err: FromUtf8Error) -> Self {
        Self::ErrorUtf8Decode { error: err.to_string() }
    }
}

impl From<DecryptionError> for VodozemacError {
    fn from(err: DecryptionError) -> Self {
        Self::ErrorDecryption { error: err.to_string() }
    }
}