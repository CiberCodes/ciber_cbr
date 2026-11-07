//! Demonstrates authenticated encryption using AES-256-GCM.

use ciber_cbr::crypto::{decrypt, encrypt, generate_key};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = generate_key()?;

    let plaintext = b"Confidential CBR data";
    let aad = b"ciber-cbr-example";

    let encrypted = encrypt(&key, plaintext, aad)?;

    println!("Encrypted size: {} bytes", encrypted.len());

    let decrypted = decrypt(&key, &encrypted, aad)?;

    assert_eq!(decrypted, plaintext);

    println!("Decryption successful.");
    println!("Original data: {}", String::from_utf8(decrypted)?);

    Ok(())
}
