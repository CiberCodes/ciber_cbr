//! Cryptographic error types and result definitions.
//!
//! This module provides a dedicated error type for cryptographic
//! operations, keeping cryptographic failures independent of
//! the general CBR error handling system.

use std::fmt;

/// Result type used by cryptographic operations.
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Errors produced by cryptographic operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    /// The supplied key has an invalid length.
    InvalidKey,

    /// The supplied nonce has an invalid length.
    InvalidNonce,

    /// The supplied authentication tag is invalid.
    InvalidTag,

    /// Encryption failed.
    EncryptionFailed,

    /// Decryption failed or authentication verification failed.
    DecryptionFailed,

    /// Key derivation failed.
    KeyDerivationFailed,

    /// Hash verification failed.
    HashVerificationFailed,

    /// The supplied input is invalid.
    InvalidInput,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidKey => "invalid cryptographic key",
            Self::InvalidNonce => "invalid cryptographic nonce",
            Self::InvalidTag => "invalid authentication tag",
            Self::EncryptionFailed => "encryption failed",
            Self::DecryptionFailed => {
                "decryption failed or authentication verification failed"
            }
            Self::KeyDerivationFailed => "key derivation failed",
            Self::HashVerificationFailed => "hash verification failed",
            Self::InvalidInput => "invalid cryptographic input",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for CryptoError {}
