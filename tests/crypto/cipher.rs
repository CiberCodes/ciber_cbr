//! Tests for authenticated encryption and decryption.
use ciber_cbr::crypto::{CryptoError, NONCE_SIZE, TAG_SIZE, decrypt, encrypt};

#[test]
fn encrypt_and_decrypt_round_trip() {
    let key = [42u8; 32];
    let plaintext = b"Ciber CBR encryption";
    let aad = b"application metadata";

    let encrypted = match encrypt(&key, plaintext, aad) {
        Ok(value) => value,
        Err(error) => panic!("encryption failed: {error}"),
    };

    let decrypted = match decrypt(&key, &encrypted, aad) {
        Ok(value) => value,
        Err(error) => panic!("decryption failed: {error}"),
    };

    assert_eq!(decrypted, plaintext);
}

#[test]
fn encrypt_generates_nonce() {
    let key = [42u8; 32];

    let encrypted = match encrypt(&key, b"CBR", b"") {
        Ok(value) => value,
        Err(error) => panic!("encryption failed: {error}"),
    };

    assert!(encrypted.len() >= NONCE_SIZE + TAG_SIZE);
}

#[test]
fn encrypt_generates_different_outputs() {
    let key = [42u8; 32];

    let first = match encrypt(&key, b"CBR", b"") {
        Ok(value) => value,
        Err(error) => panic!("encryption failed: {error}"),
    };

    let second = match encrypt(&key, b"CBR", b"") {
        Ok(value) => value,
        Err(error) => panic!("encryption failed: {error}"),
    };

    assert_ne!(first, second);
}

#[test]
fn decrypt_rejects_invalid_key() {
    let result = decrypt(&[1u8; 16], &[0u8; 28], b"");

    assert_eq!(result, Err(CryptoError::InvalidKey));
}

#[test]
fn decrypt_rejects_short_input() {
    let result = decrypt(&[42u8; 32], &[0u8; 10], b"");

    assert_eq!(result, Err(CryptoError::InvalidInput));
}

#[test]
fn decrypt_rejects_modified_ciphertext() {
    let key = [42u8; 32];

    let mut encrypted = match encrypt(&key, b"CBR", b"") {
        Ok(value) => value,
        Err(error) => panic!("encryption failed: {error}"),
    };

    encrypted[NONCE_SIZE] ^= 1;

    assert_eq!(
        decrypt(&key, &encrypted, b""),
        Err(CryptoError::DecryptionFailed),
    );
}

#[test]
fn decrypt_rejects_wrong_aad() {
    let key = [42u8; 32];

    let encrypted = match encrypt(&key, b"CBR", b"original") {
        Ok(value) => value,
        Err(error) => panic!("encryption failed: {error}"),
    };

    assert_eq!(
        decrypt(&key, &encrypted, b"modified"),
        Err(CryptoError::DecryptionFailed),
    );
}
