//! Tests for cryptographic hashing and key derivation.

mod key;

use ciber_cbr::crypto::{
    CryptoError, SHA256_OUTPUT_SIZE, sha256, verify_sha256,
};

#[test]
fn sha256_returns_expected_digest() {
    let digest = sha256(b"abc");

    let expected = [
        0xBA, 0x78, 0x16, 0xBF, 0x8F, 0x01, 0xCF, 0xEA, 0x41, 0x41, 0x40, 0xDE,
        0x5D, 0xAE, 0x22, 0x23, 0xB0, 0x03, 0x61, 0xA3, 0x96, 0x17, 0x7A, 0x9C,
        0xB4, 0x10, 0xFF, 0x61, 0xF2, 0x00, 0x15, 0xAD,
    ];

    assert_eq!(digest, expected);
}

#[test]
fn sha256_returns_32_bytes() {
    assert_eq!(sha256(b"CBR").len(), SHA256_OUTPUT_SIZE);
}

#[test]
fn verify_sha256_accepts_matching_digest() {
    let digest = sha256(b"Ciber CBR");

    assert_eq!(verify_sha256(b"Ciber CBR", &digest), Ok(()));
}

#[test]
fn verify_sha256_rejects_different_data() {
    let digest = sha256(b"Ciber CBR");

    assert_eq!(
        verify_sha256(b"Different data", &digest),
        Err(CryptoError::HashVerificationFailed),
    );
}
