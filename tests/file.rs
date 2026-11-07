//! Integration tests for CBR document file handling.

use std::{
  collections::BTreeSet,
  io::{Cursor, Error},
  str::FromStr,
};

use ciber_cbr::ParseLimits;
use ciber_cbr::{Value, file::CbrFile};

#[test]
fn from_reader_parses_document() -> Result<(), Box<dyn std::error::Error>> {
  let content = "name = \"Cirious\", version = 1";
  let document = CbrFile::from_reader(Cursor::new(content))?;

  let object = document
    .value()
    .as_object()
    .ok_or_else(|| Error::other("Expected an object"))?;

  let name =
    object.get("name").ok_or_else(|| Error::other("Missing name property"))?;

  let version = object
    .get("version")
    .ok_or_else(|| Error::other("Missing version property"))?;

  assert_eq!(name.as_str(), Some("Cirious"));
  assert_eq!(version.as_i64(), Some(1));

  Ok(())
}

#[test]
fn from_reader_rejects_invalid_document() {
  let content = "name =";

  assert!(CbrFile::from_reader(Cursor::new(content)).is_err());
}

#[test]
fn open_parses_file() -> Result<(), Box<dyn std::error::Error>> {
  use std::fs;

  let path = std::env::temp_dir().join("ciber_cbr_test.cbr");

  eprintln!("{path:?}");

  fs::write(&path, "name = \"Cirious\", version = 1")?;

  let document = CbrFile::open(&path)?;

  let object = document
    .value()
    .as_object()
    .ok_or_else(|| Error::other("Expected an object"))?;

  assert_eq!(
    object.get("name").and_then(|value| value.as_str()),
    Some("Cirious")
  );

  fs::remove_file(path)?;

  Ok(())
}

#[test]
fn write_to_produces_compact_document() -> Result<(), Box<dyn std::error::Error>>
{
  let content = "name = \"Cirious\", version = 1";

  let document = CbrFile::from_reader(Cursor::new(content))?;
  let mut output = Vec::new();
  document.write_to(&mut output)?;

  let output = String::from_utf8(output)?;

  assert_eq!(output, content);

  Ok(())
}

#[test]
fn write_pretty_to_produces_formatted_document()
-> Result<(), Box<dyn std::error::Error>> {
  let content = "name = \"Cirious\", version = 1";
  let document = CbrFile::from_reader(Cursor::new(content))?;

  let mut output = Vec::new();

  document.write_pretty_to(&mut output)?;

  let output = String::from_utf8(output)?;

  assert_ne!(output, "");
  assert!(output.contains("name"));
  assert!(output.contains("version"));
  assert!(output.contains('\n'));

  Ok(())
}

#[test]
fn save_writes_document_to_file() -> Result<(), Box<dyn std::error::Error>> {
  use std::fs;

  let content = "name = \"Cirious\", version = 1";
  let document = CbrFile::from_reader(Cursor::new(content))?;

  let path = std::env::temp_dir().join("ciber_cbr_save_test.cbr");

  document.save(&path)?;

  let saved_content = fs::read_to_string(&path)?;

  assert_eq!(saved_content, content);

  fs::remove_file(path)?;

  Ok(())
}

#[test]
fn save_pretty_writes_formatted_document()
-> Result<(), Box<dyn std::error::Error>> {
  use std::fs;

  let content = "name = \"Cirious\", version = 1";
  let document = CbrFile::from_reader(Cursor::new(content))?;

  let path = std::env::temp_dir().join("ciber_cbr_save_pretty_test.cbr");

  document.save_pretty(&path)?;

  let saved_content = fs::read_to_string(&path)?;

  assert_ne!(saved_content, "");
  assert!(saved_content.contains('\n'));

  fs::remove_file(path)?;

  Ok(())
}

#[test]
#[cfg(feature = "crypto")]
fn save_pretty_refuses_plaintext_when_crypto_policy_is_enabled()
-> Result<(), Box<dyn std::error::Error>> {
  use std::fs;

  let secret = "cbr-secret-regression-marker";
  let document =
    CbrFile::from_reader(Cursor::new(format!(r#"secret = "{secret}""#)))?;
  let mut document = document;
  document.set_crypto(true)?;
  let path =
    std::env::temp_dir().join("ciber_cbr_crypto_policy_pretty_test.cbr");
  let _ = fs::remove_file(&path);

  assert!(document.save_pretty(&path).is_err());
  assert!(!path.exists());

  Ok(())
}

#[test]
#[cfg(not(feature = "crypto"))]
fn crypto_policy_cannot_be_enabled_without_feature() {
  let mut document = CbrFile::new();
  let error = document.set_crypto(true).unwrap_err();
  assert_eq!(error.kind(), ciber_cbr::ErrorKind::InvalidValue);
  assert!(!document.options().crypto);
}

#[test]
fn parse_limits_enforce_input_string_depth_and_collection_boundaries() {
  let limits = ParseLimits {
    max_input_bytes: 20,
    max_depth: 2,
    max_collection_items: 2,
    max_string_bytes: 3,
  };
  assert!(CbrFile::parse_with_limits("x = [1, 2]", limits).is_ok());
  assert!(CbrFile::parse_with_limits("x = [[1]]", limits).is_ok());
  assert!(CbrFile::parse_with_limits("x = [1, 2, 3]", limits).is_err());
  assert!(CbrFile::parse_with_limits("x = [[[1]]]", limits).is_err());
  assert!(CbrFile::parse_with_limits("x = \"four\"", limits).is_err());
  assert!(CbrFile::parse_with_limits("x = \"éé\"", limits).is_err());
  assert!(
    CbrFile::parse_with_limits("x = (a = 1, b = 2, c = 3)", limits).is_err()
  );
  assert!(
    CbrFile::parse_with_limits("x = 12345678901234567890", limits).is_err()
  );
}

#[test]
fn reader_enforces_input_limit_before_unbounded_read() {
  let limits = ParseLimits { max_input_bytes: 5, ..ParseLimits::default() };
  assert!(
    CbrFile::from_reader_with_limits(Cursor::new("name = 1"), limits).is_err()
  );
}

#[test]
#[cfg(feature = "crypto")]
fn encrypted_save_round_trips_document_without_plaintext_leakage()
-> Result<(), Box<dyn std::error::Error>> {
  use std::fs;

  let secret = "cbr-encrypted-secret-marker";
  let mut document =
    CbrFile::from_reader(Cursor::new(format!(r#"secret = "{secret}""#)))?;
  document.set_crypto(true)?;
  let key = ciber_cbr::crypto::generate_key()?;
  let path = std::env::temp_dir().join("ciber_cbr_encrypted_save_test.cbr");
  document.save_encrypted(&path, &key, b"file-test")?;

  let ciphertext = fs::read(&path)?;
  assert!(!ciphertext.windows(secret.len()).any(|w| w == secret.as_bytes()));
  let plaintext = ciber_cbr::crypto::decrypt(&key, &ciphertext, b"file-test")?;
  assert!(String::from_utf8(plaintext)?.contains(secret));

  fs::remove_file(path)?;
  Ok(())
}

#[test]
fn compact_round_trip_preserves_document()
-> Result<(), Box<dyn std::error::Error>> {
  let content = "name = \"Cirious\", version = 1";
  let original = CbrFile::from_reader(Cursor::new(content))?;

  let mut output = Vec::new();
  original.write_to(&mut output)?;

  let restored = CbrFile::from_reader(Cursor::new(output))?;

  assert_eq!(original.value(), restored.value());

  Ok(())
}

#[test]
fn pretty_round_trip_preserves_document()
-> Result<(), Box<dyn std::error::Error>> {
  let content = "name = \"Cirious\", version = 1";

  let original = CbrFile::from_reader(Cursor::new(content))?;

  let mut output = Vec::new();
  original.write_pretty_to(&mut output)?;

  let restored = CbrFile::from_reader(Cursor::new(output))?;

  assert_eq!(original.value(), restored.value());

  Ok(())
}

#[test]
fn is_empty_returns_true_for_empty_object() {
  assert!(CbrFile::new().is_empty());
}

#[test]
fn is_empty_returns_false_for_non_empty_object() {
  let mut document = CbrFile::new();

  match document.value_mut().insert("name", Value::from("Cirious")) {
    Ok(_) => {}
    Err(value) => panic!("expected an object, got {value:?}"),
  }

  assert!(!document.is_empty());
}

#[test]
fn is_empty_returns_false_for_non_object_values() {
  assert!(!CbrFile::from_value(Value::Null).is_empty());
  assert!(!CbrFile::from_value(Value::Array(Vec::new())).is_empty());
  assert!(!CbrFile::from_value(Value::String(String::new())).is_empty());
  assert!(!CbrFile::from_value(Value::Set(BTreeSet::new())).is_empty());
}

#[test]
fn should_save_and_read_root_object_without_parentheses()
-> Result<(), Box<dyn std::error::Error>> {
  let mut value = Value::empty_object();

  let _ = value.insert("enabled", Value::from(true));
  let _ = value.insert("name", Value::from("CiberApp"));
  let _ = value.insert("version", Value::from(1_u64));

  let document = CbrFile::from_value(value);

  let mut output = Vec::new();

  document.write_to(&mut output)?;

  let content = String::from_utf8(output)?;

  assert_eq!(content, "enabled = true, name = \"CiberApp\", version = 1");

  let parsed = CbrFile::from_str(&content)?;

  assert_eq!(parsed.value().get("enabled"), document.value().get("enabled"),);

  assert_eq!(parsed.value().get("name"), document.value().get("name"),);

  assert_eq!(parsed.value().get("version").and_then(Value::as_u64), Some(1),);

  Ok(())
}
