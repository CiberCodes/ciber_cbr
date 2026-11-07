//! Demonstrates cryptographic key generation.

use ciber_cbr::crypto::{KEY_SIZE, generate_key};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let key = generate_key()?;

  println!("Generated key: {key:02x?}");
  println!("Key size: {KEY_SIZE} bytes");

  Ok(())
}
