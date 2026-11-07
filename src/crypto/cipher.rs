//! Authenticated encryption and decryption.
//!
//! This module provides AES-256-GCM operations for arbitrary
//! byte sequences, including optional additional authenticated data.
//!
//! Encrypted output contains the nonce followed by the ciphertext
//! and authentication tag.

use aes_gcm::{
    Aes256Gcm, KeyInit,
    aead::{Aead, Payload},
};

use super::error::{CryptoError, CryptoResult};

/// The AES-256-GCM nonce size in bytes.
pub const NONCE_SIZE: usize = 12;

/// The AES-256-GCM authentication tag size in bytes.
pub const TAG_SIZE: usize = 16;

/// Encrypts data using AES-256-GCM.
///
/// A fresh cryptographically secure nonce is generated for every
/// encryption operation. The returned buffer contains the nonce
/// followed by the ciphertext and authentication tag.
///
/// # Arguments
///
/// - `key`: The 256-bit encryption key.
/// - `plaintext`: The data to encrypt.
/// - `aad`: Additional authenticated data that is not encrypted.
///
/// # Returns
///
/// A byte vector containing the nonce, ciphertext, and authentication tag.
///
/// # Errors
///
/// Returns an error if the key is invalid, nonce generation fails,
/// or encryption cannot be completed.
pub fn encrypt(
    key: &[u8],
    plaintext: &[u8],
    aad: &[u8],
) -> CryptoResult<Vec<u8>> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKey)?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];

    getrandom::fill(&mut nonce_bytes)
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let nonce = aes_gcm::Nonce::from(nonce_bytes);

    let payload = Payload { msg: plaintext, aad };

    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());

    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypts data previously encrypted using AES-256-GCM.
///
/// The input must contain the nonce followed by the ciphertext
/// and authentication tag, as produced by [`encrypt`].
///
/// # Arguments
///
/// - `key`: The 256-bit encryption key.
/// - `ciphertext`: The encrypted buffer containing the nonce.
/// - `aad`: The additional authenticated data used during encryption.
///
/// # Returns
///
/// The original plaintext.
///
/// # Errors
///
/// Returns an error if the key is invalid, the input is too short,
/// or authentication verification fails.
pub fn decrypt(
    key: &[u8],
    ciphertext: &[u8],
    aad: &[u8],
) -> CryptoResult<Vec<u8>> {
    if ciphertext.len() < NONCE_SIZE + TAG_SIZE {
        return Err(CryptoError::InvalidInput);
    }

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKey)?;

    let (nonce_bytes, encrypted_data) = ciphertext.split_at(NONCE_SIZE);

    let nonce_array: [u8; NONCE_SIZE] =
        nonce_bytes.try_into().map_err(|_| CryptoError::InvalidNonce)?;

    let nonce = aes_gcm::Nonce::from(nonce_array);

    let payload = Payload { msg: encrypted_data, aad };

    cipher.decrypt(&nonce, payload).map_err(|_| CryptoError::DecryptionFailed)
}
