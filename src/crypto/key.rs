//! Cryptographic key generation and derivation.
//!
//! This module provides secure random key generation and
//! key derivation using HKDF-SHA256.

use hkdf::Hkdf;
use sha2::Sha256;

use super::error::{CryptoError, CryptoResult};

/// The default cryptographic key size in bytes.
pub const KEY_SIZE: usize = 32;

/// Generates a cryptographically secure random key.
///
/// The key is generated using the operating system's
/// cryptographically secure random number generator.
///
/// # Returns
///
/// A randomly generated 256-bit key.
///
/// # Errors
///
/// Returns [`CryptoError::InvalidInput`] if the random
/// number generator fails.
pub fn generate_key() -> CryptoResult<[u8; KEY_SIZE]> {
  let mut key = [0u8; KEY_SIZE];

  getrandom::fill(&mut key).map_err(|_| CryptoError::InvalidInput)?;

  Ok(key)
}

/// Derives a cryptographic key using HKDF-SHA256.
///
/// # Arguments
///
/// - `secret`: The input keying material.
/// - `salt`: An optional salt used during extraction.
/// - `info`: Context-specific information used during expansion.
///
/// # Returns
///
/// A derived 256-bit key.
///
/// # Errors
///
/// Returns [`CryptoError::KeyDerivationFailed`] if key derivation fails.
pub fn derive_key(
  secret: &[u8],
  salt: Option<&[u8]>,
  info: &[u8],
) -> CryptoResult<[u8; KEY_SIZE]> {
  if secret.is_empty() {
    return Err(CryptoError::InvalidInput);
  }

  let hkdf = Hkdf::<Sha256>::new(salt, secret);
  let mut key = [0u8; KEY_SIZE];

  hkdf.expand(info, &mut key).map_err(|_| CryptoError::KeyDerivationFailed)?;

  Ok(key)
}
