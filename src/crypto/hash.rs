//! Cryptographic hashing and hash verification.
//!
//! This module provides SHA-256 hashing operations for arbitrary
//! byte sequences and constant-time hash verification.

use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use super::error::{CryptoError, CryptoResult};

/// The output size of an SHA-256 digest in bytes.
pub const SHA256_OUTPUT_SIZE: usize = 32;

/// Computes the SHA-256 digest of the supplied data.
///
/// # Arguments
///
/// - `data`: The input data to hash.
///
/// # Returns
///
/// A 32-byte SHA-256 digest.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; SHA256_OUTPUT_SIZE] {
  let digest = Sha256::digest(data);
  let mut output = [0u8; SHA256_OUTPUT_SIZE];

  output.copy_from_slice(&digest);

  output
}

/// Verifies whether the supplied data matches the expected SHA-256 digest.
///
/// The digest comparison is performed in constant time.
///
/// # Arguments
///
/// - `data`: The input data to verify.
/// - `expected`: The expected 32-byte SHA-256 digest.
///
/// # Errors
///
/// Returns [`CryptoError::HashVerificationFailed`] if the
/// calculated digest does not match the expected digest.
pub fn verify_sha256(
  data: &[u8],
  expected: &[u8; SHA256_OUTPUT_SIZE],
) -> CryptoResult<()> {
  let actual = sha256(data);

  if bool::from(actual.ct_eq(expected)) {
    Ok(())
  } else {
    Err(CryptoError::HashVerificationFailed)
  }
}
