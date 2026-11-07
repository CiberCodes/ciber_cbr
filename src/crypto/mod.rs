//! Cryptographic functionality for CBR data.
//!
//! This module provides cryptographic primitives for hashing,
//! key generation, key derivation, and authenticated encryption.
//!
//! The implementation uses established cryptographic algorithms
//! through audited Rust cryptography crates.

mod cipher;
mod error;
mod hash;
mod key;

pub use cipher::{NONCE_SIZE, TAG_SIZE, decrypt, encrypt};
pub use error::{CryptoError, CryptoResult};
pub use hash::{SHA256_OUTPUT_SIZE, sha256, verify_sha256};
pub use key::{KEY_SIZE, derive_key, generate_key};
