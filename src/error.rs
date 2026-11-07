//! Error types and diagnostics for the CBR data format.
//!
//! This module defines the error types used throughout
//! `ciber_cbr` to report failures during lexical analysis,
//! parsing, formatting, value processing, and
//! input/output operations.
//!
//! Errors may optionally include a [`Position`] identifying
//! where a problem occurred in a CBR document.
//!
//! # Examples
//!
//! Creating and inspecting an error:
//!
//! ```
//! use ciber_cbr::{Error, ErrorKind, Position};
//!
//! let error =
//!   Error::syntax("unexpected token").with_position(Position::new(2, 5));
//!
//! assert_eq!(error.kind(), ErrorKind::Syntax);
//! assert_eq!(error.position(), Some(Position::new(2, 5)));
//! ```

use std::{error::Error as StdError, fmt, io, num::TryFromIntError};

/// The result type used throughout `ciber_cbr`.
///
/// This type alias represents the standard [`Result`] with
/// [`Error`] as its error type.
///
/// # Examples
///
/// ```
/// use ciber_cbr::Result;
///
/// fn validate(value: bool) -> Result<()> {
///   if value {
///     Ok(())
///   } else {
///     Err(ciber_cbr::Error::invalid_value("invalid value"))
///   }
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Identifies the category of an error produced while
/// processing CBR data.
///
/// Error kinds allow callers to distinguish between
/// different failure categories without inspecting
/// individual error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ErrorKind {
  /// The document contains invalid or malformed syntax.
  Syntax,

  /// The input ended before the expected content was found.
  UnexpectedEof,

  /// A value is invalid or cannot be represented by the
  /// supported CBR data model.
  InvalidValue,

  /// A value does not match the expected type.
  TypeMismatch,

  /// An error occurred while converting structured data
  /// into its CBR representation.
  Serialization,

  /// An error occurred while converting CBR data into
  /// another structured representation.
  Deserialization,

  /// An error occurred while formatting a CBR document.
  Formatting,

  /// An input or output operation failed.
  Io,
}

/// Identifies a position within a CBR document.
///
/// Line and column numbers are one-based, meaning that the
/// first character of a document is at line 1, column 1.
///
/// The caller is responsible for providing non-zero values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
  line: usize,
  column: usize,
}

impl Position {
  /// Creates a position within a document.
  ///
  /// Both line and column numbers are one-based.
  ///
  /// The caller is responsible for providing non-zero values.
  ///
  /// # Arguments
  ///
  /// - `line`: The line number within the document.
  /// - `column`: The column number within the line.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::Position;
  ///
  /// let position = Position::new(3, 8);
  ///
  /// assert_eq!(position.line(), 3);
  /// assert_eq!(position.column(), 8);
  /// ```
  #[must_use]
  pub const fn new(line: usize, column: usize) -> Self { Self { line, column } }

  /// Returns the one-based line number.
  #[must_use]
  pub const fn line(&self) -> usize { self.line }

  /// Returns the one-based column number.
  #[must_use]
  pub const fn column(&self) -> usize { self.column }
}

/// Represents an error encountered while processing CBR data.
///
/// An error contains a category, a descriptive message,
/// an optional document position, and an optional underlying
/// I/O error.
///
/// Use the provided constructors to create errors associated
/// with specific processing operations. A position can be
/// attached using [`Error::with_position`].
///
/// # Examples
///
/// ```
/// use ciber_cbr::{Error, ErrorKind, Position};
///
/// let error = Error::invalid_value("unsupported number")
///   .with_position(Position::new(1, 12));
///
/// assert_eq!(error.kind(), ErrorKind::InvalidValue);
/// assert_eq!(error.message(), "unsupported number");
/// assert_eq!(error.position(), Some(Position::new(1, 12)));
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Error {
  kind: ErrorKind,
  message: String,
  position: Option<Position>,

  #[cfg_attr(feature = "serde", serde(skip))]
  source: Option<io::Error>,
}

impl Error {
  /// Creates an error with the specified category and
  /// descriptive message.
  ///
  /// This constructor is useful when an existing error
  /// category does not have a dedicated convenience
  /// constructor.
  ///
  /// # Arguments
  ///
  /// - `kind`: The category of the error.
  /// - `message`: A description of the failure.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::{Error, ErrorKind};
  ///
  /// let error = Error::new(ErrorKind::Syntax, "unexpected token");
  ///
  /// assert_eq!(error.kind(), ErrorKind::Syntax);
  /// ```
  #[must_use]
  pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
    Self { kind, message: message.into(), position: None, source: None }
  }

  /// Creates a syntax error.
  ///
  /// Use this constructor when a document contains
  /// unexpected or malformed syntax.
  #[must_use]
  pub fn syntax(message: impl Into<String>) -> Self {
    Self::new(ErrorKind::Syntax, message)
  }

  /// Creates an unexpected end-of-input error.
  ///
  /// Use this constructor when the input ends before the
  /// parser or lexer can complete the expected operation.
  #[must_use]
  pub fn unexpected_eof(message: impl Into<String>) -> Self {
    Self::new(ErrorKind::UnexpectedEof, message)
  }

  /// Creates an invalid-value error.
  ///
  /// Use this constructor when a value is malformed,
  /// outside the supported range, or otherwise invalid.
  #[must_use]
  pub fn invalid_value(message: impl Into<String>) -> Self {
    Self::new(ErrorKind::InvalidValue, message)
  }

  /// Creates a type-mismatch error.
  ///
  /// Use this constructor when a value does not have the
  /// type required by the current operation.
  #[must_use]
  pub fn type_mismatch(message: impl Into<String>) -> Self {
    Self::new(ErrorKind::TypeMismatch, message)
  }

  /// Creates a serialization error.
  ///
  /// Use this constructor when structured data cannot be
  /// represented in the requested output format.
  #[must_use]
  pub fn serialization(message: impl Into<String>) -> Self {
    Self::new(ErrorKind::Serialization, message)
  }

  /// Creates a deserialization error.
  ///
  /// Use this constructor when input data cannot be
  /// converted into the requested structured representation.
  #[must_use]
  pub fn deserialization(message: impl Into<String>) -> Self {
    Self::new(ErrorKind::Deserialization, message)
  }

  /// Creates a formatting error.
  ///
  /// Use this constructor when a value cannot be converted
  /// into a valid textual CBR representation.
  #[must_use]
  pub fn formatting(message: impl Into<String>) -> Self {
    Self::new(ErrorKind::Formatting, message)
  }

  /// Creates an input/output error from a standard library
  /// I/O error.
  ///
  /// The original error is preserved as the underlying
  /// error source.
  ///
  /// # Arguments
  ///
  /// - `error`: The original I/O error.
  #[must_use]
  pub fn io(error: io::Error) -> Self {
    let message = error.to_string();

    Self { kind: ErrorKind::Io, message, position: None, source: Some(error) }
  }

  /// Attaches a document position to this error.
  ///
  /// The supplied position replaces any position already
  /// associated with the error.
  ///
  /// # Arguments
  ///
  /// - `position`: The location associated with the error.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::{Error, Position};
  ///
  /// let error =
  ///   Error::syntax("unexpected token").with_position(Position::new(2, 4));
  ///
  /// assert_eq!(error.position(), Some(Position::new(2, 4)));
  /// ```
  #[must_use]
  pub const fn with_position(mut self, position: Position) -> Self {
    self.position = Some(position);
    self
  }

  /// Returns the category of this error.
  #[must_use]
  pub const fn kind(&self) -> ErrorKind { self.kind }

  /// Returns the descriptive message associated with
  /// this error.
  #[must_use]
  pub fn message(&self) -> &str { &self.message }

  /// Returns the document position associated with this
  /// error, if one is available.
  #[must_use]
  pub const fn position(&self) -> Option<Position> { self.position }
}

impl fmt::Display for Error {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(position) = self.position {
      write!(
        formatter,
        "{} at line {}, column {}: {}",
        self.kind,
        position.line(),
        position.column(),
        self.message,
      )
    } else {
      write!(formatter, "{}: {}", self.kind, self.message)
    }
  }
}

impl fmt::Display for ErrorKind {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    let name = match self {
      Self::Syntax => "syntax error",
      Self::UnexpectedEof => "unexpected end of input",
      Self::InvalidValue => "invalid value",
      Self::TypeMismatch => "type mismatch",
      Self::Serialization => "serialization error",
      Self::Deserialization => "deserialization error",
      Self::Formatting => "formatting error",
      Self::Io => "I/O error",
    };

    formatter.write_str(name)
  }
}

impl StdError for Error {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    self.source.as_ref().map(|source| source as &(dyn StdError + 'static))
  }
}

impl From<io::Error> for Error {
  fn from(error: io::Error) -> Self { Self::io(error) }
}

impl From<TryFromIntError> for Error {
  fn from(value: TryFromIntError) -> Self {
    Self::formatting(value.to_string())
  }
}
