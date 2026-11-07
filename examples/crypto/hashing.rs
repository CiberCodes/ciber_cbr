//! Demonstrates SHA-256 hashing and verification.

use ciber_cbr::crypto::{SHA256_OUTPUT_SIZE, sha256, verify_sha256};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = b"Hello, CBR!";

    let digest = sha256(data);

    println!("SHA-256: {digest:02x?}");
    println!("Digest size: {SHA256_OUTPUT_SIZE} bytes");

    verify_sha256(data, &digest)?;

    println!("Hash verification successful.");

    Ok(())
}
