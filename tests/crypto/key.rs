//! Tests for cryptographic key generation and derivation.

use ciber_cbr::crypto::{CryptoError, KEY_SIZE, derive_key, generate_key};

#[test]
fn generate_key_returns_expected_size() -> Result<(), Box<dyn std::error::Error>>
{
    let key = generate_key()?;

    assert_eq!(key.len(), KEY_SIZE);

    Ok(())
}

#[test]
fn generate_key_generates_different_keys()
-> Result<(), Box<dyn std::error::Error>> {
    let first = generate_key()?;
    let second = generate_key()?;
    assert_ne!(first, second);
    Ok(())
}

#[test]
fn derive_key_returns_expected_size() -> Result<(), Box<dyn std::error::Error>>
{
    let key = derive_key(b"secret", None, b"cbr")?;

    assert_eq!(key.len(), KEY_SIZE);
    Ok(())
}

#[test]
fn derive_key_is_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let first = derive_key(b"secret", Some(b"salt"), b"cbr")?;
    let second = derive_key(b"secret", Some(b"salt"), b"cbr")?;

    assert_eq!(first, second);
    Ok(())
}

#[test]
fn derive_key_changes_with_different_context()
-> Result<(), Box<dyn std::error::Error>> {
    let first = derive_key(b"secret", Some(b"salt"), b"cbr")?;
    let second = derive_key(b"secret", Some(b"salt"), b"other")?;
    assert_ne!(first, second);
    Ok(())
}

#[test]
fn derive_key_rejects_empty_secret() {
    assert_eq!(derive_key(b"", None, b"cbr"), Err(CryptoError::InvalidInput),);
}
