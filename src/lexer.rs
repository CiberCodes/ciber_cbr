//! Lexical analysis for the CBR data format.
//!
//! This module provides the lexer responsible for converting
//! CBR source text into a sequence of [`Token`] instances.
//!
//! The lexer recognizes identifiers, string literals, numeric
//! literals, boolean and null values, structural delimiters,
//! separators, and comments.
//!
//! Each token retains its original position in the source
//! document through [`Position`], allowing subsequent
//! processing stages to produce accurate diagnostics.
//!
//! Numeric literals are preserved as strings during lexical
//! analysis. Their conversion into numeric [`Value`](crate::value::Value)
//! variants is handled by the parser.
//!
//! # Supported syntax
//!
//! - Unicode alphabetic identifiers and underscores.
//! - Hyphens and alphanumeric characters in identifiers after the first
//!   character. Identifiers cannot end with a hyphen.
//! - String literals with escape sequences.
//! - Signed integer and floating-point literals.
//! - Scientific notation using `e` or `E`.
//! - Boolean literals: `true` and `false`.
//! - Null literal: `null`.
//! - Line comments beginning with `//`.
//! - Block comments delimited by `/*` and `*/`.
//!
//! Numeric literals follow JSON-style syntax:
//!
//! - Leading zeros are not allowed, except for zero itself.
//! - A leading plus sign is not allowed.
//! - Fractional parts require at least one digit.
//! - Exponents require at least one digit.
//!
//! # Examples
//!
//! ```
//! use ciber_cbr::lexer::tokenize;
//!
//! let tokens = tokenize(r#"name = "CiberCodes", version = 1"#)?;
//!
//! assert!(!tokens.is_empty());
//! # Ok::<(), ciber_cbr::Error>(())
//! ```

use crate::error::{Error, Position, Result};

/// Represents a lexical token recognized in a CBR document.
///
/// A token consists of a [`TokenKind`] and its starting
/// [`Position`] in the original source document.
///
/// Tokens are produced by [`Lexer`] and consumed by
/// subsequent processing stages, such as the parser.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Token {
  kind: TokenKind,
  position: Position,
}

impl Token {
  /// Creates a token with the specified kind and source position.
  ///
  /// # Arguments
  ///
  /// - `kind`: The lexical category and associated token data.
  /// - `position`: The starting position of the token.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::{
  ///   Position,
  ///   lexer::{Token, TokenKind},
  /// };
  ///
  /// let token = Token::new(TokenKind::Equals, Position::new(1, 5));
  ///
  /// assert_eq!(token.position(), Position::new(1, 5));
  /// ```
  #[must_use]
  pub const fn new(kind: TokenKind, position: Position) -> Self {
    Self { kind, position }
  }

  /// Returns a reference to the token's lexical kind.
  #[must_use]
  pub const fn kind(&self) -> &TokenKind { &self.kind }

  /// Returns the token's starting position in the source document.
  #[must_use]
  pub const fn position(&self) -> Position { self.position }
}

/// Represents the lexical categories recognized by the CBR lexer.
///
/// Variants containing data retain the value extracted from
/// the source document. Numeric literals, in particular, are
/// stored as strings to preserve their original representation
/// until they are interpreted by the parser.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TokenKind {
  /// An identifier used as a property name or another
  /// identifier in the CBR document.
  Identifier(String),

  /// A string literal containing its decoded UTF-8 contents.
  String(String),

  /// A numeric literal preserved in its original textual form.
  ///
  /// Keeping the original representation avoids losing
  /// precision during lexical analysis.
  Number(String),

  /// A boolean literal.
  Bool(bool),

  /// The null literal.
  Null,

  /// An opening curly brace (`{`), used to delimit a set.
  LeftBrace,

  /// A closing curly brace (`}`), used to delimit a set.
  RightBrace,

  /// An opening square bracket (`[`), used to delimit an array.
  LeftBracket,

  /// A closing square bracket (`]`), used to delimit an array.
  RightBracket,

  /// An opening parenthesis (`(`), used to delimit an object.
  LeftParen,

  /// A closing parenthesis (`)`), used to delimit an object.
  RightParen,

  /// An equals sign `=`.
  Equals,

  /// A comma `,` used to separate properties and collection values.
  Comma,

  /// A line break used as a separator between properties.
  Newline,

  /// Marks the end of the source input.
  Eof,
}

/// Converts CBR source text into lexical tokens.
///
/// The lexer stores a character copy of the complete input, so memory use
/// is proportional to the source size. Apply an input-size limit at the
/// application boundary when processing untrusted documents.
///
/// The lexer tracks its current character index and
/// one-based line and column positions while traversing
/// the source.
///
/// It recognizes literals, identifiers, structural
/// delimiters, separators, whitespace, and comments.
///
/// Numeric literals are preserved as strings rather than
/// converted into numeric values.
///
/// Use [`Lexer::tokenize`] to process the complete document
/// or [`tokenize`] as a convenience function.
///
/// # Examples
///
/// ```
/// use ciber_cbr::lexer::Lexer;
///
/// let tokens = Lexer::new("enabled = true").tokenize()?;
///
/// assert_eq!(tokens.len(), 4);
/// # Ok::<(), ciber_cbr::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Lexer {
  characters: Vec<char>,
  index: usize,
  line: usize,
  column: usize,
  previous_was_carriage_return: bool,
}

impl Lexer {
  /// Creates a lexer for the supplied CBR source document.
  ///
  /// The lexer starts at the first character, with its position
  /// initialized to line 1, column 1.
  ///
  /// # Arguments
  ///
  /// - `source`: The CBR source text to tokenize.
  #[must_use]
  pub fn new(source: &str) -> Self {
    Self {
      characters: source.chars().collect(),
      index: 0,
      line: 1,
      column: 1,
      previous_was_carriage_return: false,
    }
  }

  /// Converts the complete source document into a sequence
  /// of lexical tokens.
  ///
  /// Whitespace and comments are skipped. An [`TokenKind::Eof`]
  /// token is appended to the resulting sequence to mark
  /// the end of the input.
  ///
  /// # Errors
  ///
  /// Returns an error if the source contains an unexpected
  /// character, an invalid string escape, a malformed numeric
  /// literal, or an unterminated string or block comment.
  pub fn tokenize(mut self) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();

    while !self.is_at_end() {
      self.skip_whitespace_and_comments()?;

      if self.is_at_end() {
        break;
      }

      if matches!(self.peek(), Some('\n' | '\r')) {
        let position = self.position();

        self.advance();

        if self.characters.get(self.index.saturating_sub(1)) == Some(&'\r')
          && self.peek() == Some('\n')
        {
          self.advance();
        }

        tokens.push(Token::new(TokenKind::Newline, position));

        continue;
      }

      tokens.push(self.next_token()?);
    }

    tokens.push(Token::new(TokenKind::Eof, self.position()));

    Ok(tokens)
  }

  fn next_token(&mut self) -> Result<Token> {
    let position = self.position();

    let character = self.advance_required("expected token", position)?;

    let kind = match character {
      '{' => TokenKind::LeftBrace,
      '}' => TokenKind::RightBrace,
      '[' => TokenKind::LeftBracket,
      ']' => TokenKind::RightBracket,
      '(' => TokenKind::LeftParen,
      ')' => TokenKind::RightParen,
      '=' => TokenKind::Equals,
      ',' => TokenKind::Comma,
      '"' => TokenKind::String(self.read_string(position)?),

      '-' | '0'..='9' => {
        TokenKind::Number(self.read_number(character, position)?)
      }

      character if Self::is_identifier_start(character) => {
        self.read_identifier(character)?
      }

      character => {
        return Err(
          Error::syntax(format!("unexpected character: {character:?}"))
            .with_position(position),
        );
      }
    };

    Ok(Token::new(kind, position))
  }

  fn read_identifier(&mut self, first: char) -> Result<TokenKind> {
    let mut identifier = String::from(first);

    while let Some(character) = self.peek() {
      if !Self::is_identifier_continue(character) {
        break;
      }

      if let Some(character) = self.advance() {
        identifier.push(character);
      }
    }

    if identifier.ends_with('-') {
      return Err(
        Error::syntax("identifiers cannot end with a hyphen")
          .with_position(self.position()),
      );
    }

    let kind = match identifier.as_str() {
      "true" => TokenKind::Bool(true),
      "false" => TokenKind::Bool(false),
      "null" => TokenKind::Null,
      _ => TokenKind::Identifier(identifier),
    };

    Ok(kind)
  }

  fn read_string(&mut self, position: Position) -> Result<String> {
    let mut value = String::new();

    while let Some(character) = self.advance() {
      match character {
        '"' => return Ok(value),

        '\\' => {
          let escape_position = self.position();

          let escaped = self.advance().ok_or_else(|| {
            Error::unexpected_eof("expected an escape sequence")
              .with_position(escape_position)
          })?;

          match escaped {
            '"' => value.push('"'),
            '\\' => value.push('\\'),
            'n' => value.push('\n'),
            'r' => value.push('\r'),
            't' => value.push('\t'),
            '0' => value.push('\0'),

            character => {
              return Err(
                Error::syntax(format!(
                  "invalid escape sequence: \\{character}"
                ))
                .with_position(escape_position),
              );
            }
          }
        }

        '\n' | '\r' => {
          return Err(
            Error::syntax("newline inside string literal")
              .with_position(position),
          );
        }

        character => value.push(character),
      }
    }

    Err(
      Error::unexpected_eof("unterminated string literal")
        .with_position(position),
    )
  }

  fn read_integer_part(
    &mut self,
    first: char,
    position: Position,
    number: &mut String,
  ) -> Result<()> {
    if first == '0' {
      if self.peek().is_some_and(|character| character.is_ascii_digit()) {
        return Err(
          Error::syntax("leading zeros are not allowed in numeric literals")
            .with_position(position),
        );
      }

      return Ok(());
    }

    while self.peek().is_some_and(|character| character.is_ascii_digit()) {
      number.push(self.advance_required("expected a digit", self.position())?);
    }

    Ok(())
  }

  fn read_fractional_part(&mut self, number: &mut String) -> Result<()> {
    if self.peek() != Some('.') {
      return Ok(());
    }

    number.push(
      self.advance_required("expected a decimal point", self.position())?,
    );

    if !self.peek().is_some_and(|character| character.is_ascii_digit()) {
      return Err(
        Error::syntax("expected a digit after decimal point")
          .with_position(self.position()),
      );
    }

    while self.peek().is_some_and(|character| character.is_ascii_digit()) {
      number.push(self.advance_required("expected a digit", self.position())?);
    }

    Ok(())
  }

  fn read_exponent_part(&mut self, number: &mut String) -> Result<()> {
    if !self.peek().is_some_and(|character| matches!(character, 'e' | 'E')) {
      return Ok(());
    }

    number
      .push(self.advance_required("expected an exponent", self.position())?);

    if self.peek().is_some_and(|character| matches!(character, '+' | '-')) {
      number.push(
        self.advance_required("expected an exponent sign", self.position())?,
      );
    }

    if !self.peek().is_some_and(|character| character.is_ascii_digit()) {
      return Err(
        Error::syntax("expected a digit in exponent")
          .with_position(self.position()),
      );
    }

    while self.peek().is_some_and(|character| character.is_ascii_digit()) {
      number.push(self.advance_required("expected a digit", self.position())?);
    }

    Ok(())
  }

  fn read_number(&mut self, first: char, position: Position) -> Result<String> {
    let mut number = String::from(first);

    if first == '-'
      && !self.peek().is_some_and(|character| character.is_ascii_digit())
    {
      return Err(
        Error::syntax("expected a digit after '-'").with_position(position),
      );
    }

    self.read_integer_part(first, position, &mut number)?;
    self.read_fractional_part(&mut number)?;
    self.read_exponent_part(&mut number)?;

    if self.peek().is_some_and(Self::is_identifier_start) {
      return Err(
        Error::syntax("numeric literals cannot be followed by an identifier")
          .with_position(self.position()),
      );
    }

    Ok(number)
  }

  fn skip_whitespace_and_comments(&mut self) -> Result<()> {
    loop {
      while self.peek().is_some_and(|character| {
        character.is_whitespace() && !matches!(character, '\n' | '\r')
      }) {
        self.advance();
      }

      if self.peek() != Some('/') {
        return Ok(());
      }

      match self.peek_next() {
        Some('/') => self.skip_line_comment(),
        Some('*') => self.skip_block_comment()?,
        _ => return Ok(()),
      }
    }
  }

  fn skip_line_comment(&mut self) {
    while let Some(character) = self.peek() {
      if matches!(character, '\n' | '\r') {
        break;
      }

      self.advance();
    }
  }

  fn skip_block_comment(&mut self) -> Result<()> {
    let position = self.position();

    self.advance();
    self.advance();

    while !self.is_at_end() {
      if self.peek() == Some('*') && self.peek_next() == Some('/') {
        self.advance();
        self.advance();

        return Ok(());
      }

      self.advance();
    }

    Err(
      Error::unexpected_eof("unterminated block comment")
        .with_position(position),
    )
  }

  const fn position(&self) -> Position { Position::new(self.line, self.column) }

  /// Consumes the next character or returns an unexpected
  /// end-of-input error.
  ///
  /// The supplied position is attached to the error if no
  /// character remains.
  ///
  /// # Errors
  ///
  /// Returns an unexpected end-of-input error if the input
  /// has no remaining characters.
  fn advance_required(
    &mut self,
    message: &'static str,
    position: Position,
  ) -> Result<char> {
    self
      .advance()
      .ok_or_else(|| Error::unexpected_eof(message).with_position(position))
  }

  fn advance(&mut self) -> Option<char> {
    let character = self.characters.get(self.index).copied()?;

    self.index += 1;

    match character {
      '\r' => {
        self.line += 1;
        self.column = 1;
        self.previous_was_carriage_return = true;
      }

      '\n' => {
        if !self.previous_was_carriage_return {
          self.line += 1;
        }

        self.column = 1;
        self.previous_was_carriage_return = false;
      }

      _ => {
        self.column += 1;
        self.previous_was_carriage_return = false;
      }
    }

    Some(character)
  }

  fn peek(&self) -> Option<char> { self.characters.get(self.index).copied() }

  fn peek_next(&self) -> Option<char> {
    self.characters.get(self.index + 1).copied()
  }

  const fn is_at_end(&self) -> bool { self.index >= self.characters.len() }

  fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_alphabetic()
  }

  fn is_identifier_continue(character: char) -> bool {
    character == '_' || character.is_alphanumeric() || character == '-'
  }
}

/// Tokenizes a CBR source document.
///
/// This is a convenience function that creates a [`Lexer`]
/// and processes the complete source document.
///
/// The returned token sequence always ends with
/// [`TokenKind::Eof`].
///
/// # Arguments
///
/// - `source`: The CBR source text.
///
/// # Errors
///
/// Returns an error if the source contains invalid characters,
/// malformed literals, invalid escape sequences, or
/// unterminated strings or comments.
///
/// # Examples
///
/// ```
/// use ciber_cbr::lexer::{
///     tokenize,
///     TokenKind,
/// };
///
/// let tokens = tokenize(r#"name = "Cirious""#)?;
///
/// assert!(matches!(
///     tokens[0].kind(),
///     TokenKind::Identifier(name) if name == "name"
/// ));
///
/// assert!(matches!(
///     tokens[1].kind(),
///     TokenKind::Equals
/// ));
///
/// assert!(matches!(
///     tokens[2].kind(),
///     TokenKind::String(value) if value == "Cirious"
/// ));
/// # Ok::<(), ciber_cbr::Error>(())
/// ```
pub fn tokenize(source: &str) -> Result<Vec<Token>> {
  Lexer::new(source).tokenize()
}
