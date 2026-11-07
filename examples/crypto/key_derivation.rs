//! Demonstrates cryptographic key derivation using HKDF-SHA256.

use ciber_cbr::crypto::{KEY_SIZE, derive_key};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret = b"example input key material";
    let salt = b"ciber-cbr-example-salt";
    let info = b"encryption-key";

    let key = derive_key(secret, Some(salt), info)?;

    println!("Derived key size: {KEY_SIZE} bytes");
    println!("Derived key: {key:02x?}");

    Ok(())
}
