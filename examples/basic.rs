//! Demonstrates parsing a CBR document.

use ciber_cbr::crypto::{
  decrypt, encrypt, generate_key, sha256, verify_sha256,
};

fn main() -> ciber_cbr::crypto::CryptoResult<()> {
  let key = generate_key()?;

  let plaintext = b"Hello, CBR!";
  let aad = b"ciber_cbr";

  let encrypted = encrypt(&key, plaintext, aad)?;
  let decrypted = decrypt(&key, &encrypted, aad)?;

  assert_eq!(decrypted, plaintext);

  let digest = sha256(plaintext);
  verify_sha256(plaintext, &digest)?;

  Ok(())
}
