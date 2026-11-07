//! CBR document and file management.
//!
//! This module provides the high-level interface for representing and
//! manipulating CBR documents.
//!
//! A CBR document is represented internally by [`Value`],
//! allowing it to contain primitive values, arrays, sets, and objects.
//!
//! Parsing and formatting are delegated to the [`parser`](crate::parser)
//! and [`formatter`] modules, respectively.
//!
//! # Design
//!
//! [`CbrFile`] is independent of any specific application domain. It can
//! represent configuration files, structured data documents, and other
//! content supported by the CBR value model.

use std::{
  collections::BTreeMap,
  fs::File,
  io::{Read, Write},
  path::Path,
  str::FromStr,
};

use crate::{
  Error,
  error::Result,
  formatter,
  lexer::tokenize_with_limit,
  parser::{ParseLimits, Parser},
  value::Value,
};

/// A CBR document.
///
/// `CbrFile` provides a high-level interface for representing and
/// manipulating structured CBR data.
///
/// It is not restricted to configuration data and can contain any value
/// supported by [`Value`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CbrFile {
  value: Value,
  options: Options,
}

/// Preferences controlling how a [`CbrFile`] is saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Options {
  /// Whether saved CBR text should be pretty-printed.
  pub pretty: bool,
  /// Whether saving should encrypt the serialized document.
  /// Requires the crate's `crypto` feature and an encryption key.
  pub crypto: bool,
}

impl CbrFile {
  /// Creates an empty CBR document.
  ///
  /// The document is initialized with an empty object.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::file::CbrFile;
  ///
  /// let file = CbrFile::new();
  /// assert!(file.value().is_object());
  /// ```
  #[must_use]
  pub const fn new() -> Self {
    Self {
      value: Value::empty_object(),
      options: Options { pretty: false, crypto: false },
    }
  }

  /// Creates a CBR document from an existing value.
  ///
  /// # Arguments
  ///
  /// - `value`: The value to use as document content.
  #[must_use]
  pub const fn from_value(value: Value) -> Self {
    Self { value, options: Options { pretty: false, crypto: false } }
  }

  /// Returns a reference to the document content.
  #[must_use]
  pub const fn value(&self) -> &Value { &self.value }

  /// Returns the document save preferences.
  #[must_use]
  pub const fn options(&self) -> &Options { &self.options }

  /// Returns mutable access to the document save preferences.
  #[must_use]
  pub const fn options_mut(&mut self) -> &mut Options { &mut self.options }

  /// Sets whether saved text uses pretty formatting.
  pub const fn set_pretty(&mut self, enabled: bool) {
    self.options.pretty = enabled;
  }

  /// Sets whether document saving uses encryption.
  ///
  /// Returns an error if encryption is enabled without the `crypto` feature.
  ///
  /// # Errors
  ///
  /// Returns an error when `enabled` is `true` and the crate was built without
  /// the `crypto` feature.
  #[allow(clippy::missing_const_for_fn)] // Error construction is not const on the no-crypto path.
  pub fn set_crypto(&mut self, enabled: bool) -> Result<()> {
    #[cfg(not(feature = "crypto"))]
    if enabled {
      return Err(Error::invalid_value(
        "Crypto option requires the `crypto` feature",
      ));
    }
    self.options.crypto = enabled;
    Ok(())
  }

  /// Returns a mutable reference to the document content.
  #[must_use]
  pub const fn value_mut(&mut self) -> &mut Value { &mut self.value }

  /// Consumes the document and returns its inner value.
  ///
  /// This operation moves the document content without cloning it.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::{file::CbrFile, value::Value};
  ///
  /// let document = CbrFile::new();
  /// let value = document.into_inner();
  ///
  /// assert_eq!(value, Value::empty_object());
  /// ```
  #[must_use]
  pub fn into_inner(self) -> Value { self.value }

  /// Returns whether the document contains an empty object.
  ///
  /// A document is considered empty when its root value is an object
  /// containing no entries. Other values, including null, empty arrays,
  /// empty strings, and empty sets, are not considered empty.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::{file::CbrFile, value::Value};
  ///
  /// assert!(CbrFile::new().is_empty());
  /// assert!(!CbrFile::from_value(Value::Null).is_empty());
  /// assert!(!CbrFile::from_value(Value::Array(Vec::new())).is_empty());
  /// ```
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.value.as_object().is_some_and(BTreeMap::is_empty)
  }

  /// Formats the document as a compact CBR string.
  ///
  /// # Errors
  ///
  /// Returns an error if the document contains a value that cannot
  /// be represented using the CBR syntax.
  pub fn to_string(&self) -> Result<String> {
    formatter::to_string(&self.value)
  }

  /// Formats the document as a pretty-printed CBR string.
  ///
  /// # Errors
  ///
  /// Returns an error if the document contains a value that cannot
  /// be represented using the CBR syntax.
  pub fn to_string_pretty(&self) -> Result<String> {
    formatter::to_string_pretty(&self.value)
  }

  /// Creates a CBR document by reading its content from a reader.
  ///
  /// The content is parsed using the CBR lexer and parser.
  /// The complete input is buffered in memory; callers handling untrusted
  /// input should enforce an appropriate size limit before calling this method.
  ///
  /// # Errors
  ///
  /// Returns an error if reading or parsing the content fails.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::io::Cursor;
  ///
  /// use ciber_cbr::file::CbrFile;
  ///
  /// let content = "name = \"Cirious\", version = 1";
  /// let document = CbrFile::from_reader(Cursor::new(content))?;
  ///
  /// assert_eq!(
  ///   document.value().as_object().unwrap()["name"].as_str(),
  ///   Some("Cirious")
  /// );
  /// # Ok::<(), ciber_cbr::Error>(())
  /// ```
  pub fn from_reader<R: Read>(mut reader: R) -> Result<Self> {
    Self::from_reader_with_limits(&mut reader, ParseLimits::default())
  }

  /// Reads and parses a document with explicit resource limits.
  ///
  /// # Errors
  ///
  /// Returns an error if reading fails, input exceeds its byte limit, or parsing fails.
  ///
  /// # Errors
  ///
  /// Returns an error if reading fails, input exceeds the byte limit, or parsing fails.
  pub fn from_reader_with_limits<R: Read>(
    reader: R,
    limits: ParseLimits,
  ) -> Result<Self> {
    let max_bytes = limits.max_input_bytes;
    let mut content = String::new();
    reader
      .take(max_bytes.saturating_add(1) as u64)
      .read_to_string(&mut content)?;
    if content.len() > max_bytes {
      return Err(Error::invalid_value("maximum input size exceeded"));
    }
    Self::parse_with_limits(&content, limits)
  }

  /// Parses source text using explicit resource limits.
  ///
  /// # Errors
  ///
  /// Returns an error if input exceeds a configured limit or contains invalid CBR.
  ///
  /// # Errors
  ///
  /// Returns an error if input exceeds a configured limit or contains invalid CBR.
  pub fn parse_with_limits(source: &str, limits: ParseLimits) -> Result<Self> {
    let tokens = tokenize_with_limit(source, limits.max_input_bytes)?;
    let mut parser = Parser::with_limits(tokens, limits)?;
    Ok(Self::from_value(parser.parse()?))
  }

  /// Opens and parses a CBR document from a file.
  ///
  /// # Errors
  ///
  /// Returns an error if the file cannot be opened, read, or parsed.
  pub fn open(path: impl AsRef<Path>) -> Result<Self> {
    let file = File::open(path)?;

    Self::from_reader(file)
  }

  /// Writes the document to a writer using compact formatting.
  ///
  /// # Errors
  ///
  /// Returns an error if formatting or writing the document fails.
  pub fn write_to<W: Write>(&self, writer: W) -> Result<()> {
    formatter::to_writer(&self.value, writer)
  }

  /// Writes the document to a writer using pretty formatting.
  ///
  /// # Errors
  ///
  /// Returns an error if formatting or writing the document fails.
  pub fn write_pretty_to<W: Write>(&self, writer: W) -> Result<()> {
    formatter::to_writer_pretty(&self.value, writer)
  }

  /// Saves the document to a file using compact formatting.
  ///
  /// Creates the file if it does not exist. If the file already exists,
  /// its contents are truncated before writing.
  ///
  /// # Errors
  ///
  /// Returns an error if the file cannot be created or the document
  /// cannot be formatted or written.
  pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
    #[cfg(not(feature = "crypto"))]
    if self.options.crypto {
      return Err(Error::invalid_value(
        "Crypto option requires the `crypto` feature",
      ));
    }
    #[cfg(feature = "crypto")]
    if self.options.crypto {
      return Err(Error::invalid_value(
        "Crypto saving requires an explicit key; use save_encrypted",
      ));
    }
    let file = File::create(path)?;
    if self.options.pretty {
      self.write_pretty_to(file)
    } else {
      self.write_to(file)
    }
  }

  /// Saves the document to a file using pretty formatting.
  ///
  /// Creates the file if it does not exist. If the file already exists,
  /// its contents are truncated before writing.
  ///
  /// # Errors
  ///
  /// Returns an error if the file cannot be created or the document
  /// cannot be formatted or written.
  pub fn save_pretty(&self, path: impl AsRef<Path>) -> Result<()> {
    if self.options.crypto {
      return Err(Error::invalid_value(
        "Crypto saving requires an explicit key; use save_encrypted",
      ));
    }
    let file = File::create(path)?;

    self.write_pretty_to(file)
  }

  /// Saves the document encrypted with AES-256-GCM when the `crypto` feature is enabled.
  /// The configured `pretty` preference controls plaintext formatting before encryption.
  ///
  /// # Errors
  ///
  /// Returns an error if the key is invalid, encryption fails, formatting fails,
  /// or the output file cannot be written.
  #[cfg(feature = "crypto")]
  pub fn save_encrypted(
    &self,
    path: impl AsRef<Path>,
    key: &[u8],
    aad: &[u8],
  ) -> Result<()> {
    let plaintext = if self.options.pretty {
      self.to_string_pretty()?
    } else {
      self.to_string()?
    };
    let encrypted = crate::crypto::encrypt(key, plaintext.as_bytes(), aad)
      .map_err(|err| {
        Error::invalid_value(format!("Encryption failed: {err}"))
      })?;
    std::fs::write(path, encrypted)?;
    Ok(())
  }
}

impl Default for CbrFile {
  fn default() -> Self { Self::new() }
}

impl FromStr for CbrFile {
  type Err = Error;

  /// Parses a CBR document from a string slice.
  ///
  /// # Arguments
  ///
  /// - `content`: The CBR document source.
  ///
  /// # Errors
  ///
  /// Returns an error if the input contains invalid tokens or
  /// cannot be parsed as a valid CBR document.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::str::FromStr;
  ///
  /// use ciber_cbr::file::CbrFile;
  ///
  /// let file = CbrFile::from_str("name = \"Cirious\"")?;
  /// assert!(file.value().is_object());
  ///
  /// # Ok::<(), ciber_cbr::error::Error>(())
  /// ```
  fn from_str(content: &str) -> std::result::Result<Self, Self::Err> {
    Self::parse_with_limits(content, ParseLimits::default())
  }
}
