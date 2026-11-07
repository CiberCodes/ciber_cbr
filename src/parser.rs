//! Parser for the Ciber CBR data format.
//!
//! This module provides the parser responsible for converting
//! a stream of lexical tokens into structured [`Value`] instances.
//!
//! The parser supports:
//!
//! - Null and boolean values.
//! - Signed and unsigned integers.
//! - Floating-point numbers.
//! - Strings.
//! - Nested objects enclosed in parentheses.
//! - Root objects represented by top-level properties.
//! - Arrays enclosed in square brackets.
//! - Sets enclosed in curly braces.
//! - Top-level document properties.
//!
//! Objects and document properties use equals-separated key-value
//! pairs. Collection elements and object properties are separated
//! by commas or newlines.
//!
//! Duplicate object properties replace their previous values.
//!
//! Parsing errors include source positions whenever available.

use std::collections::BTreeSet;

use crate::{
    error::{Error, ErrorKind, Result},
    lexer::{Token, TokenKind},
    value::Value,
};

/// Parses a stream of CBR tokens into structured values.
///
/// The parser consumes tokens produced by the CBR lexer and builds
/// a [`Value`] representation of the input document.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Creates a parser from a token stream.
    ///
    /// The supplied token stream must contain an EOF token.
    ///
    /// # Arguments
    ///
    /// - `tokens`: The token stream produced by the CBR lexer.
    ///
    /// # Panics
    ///
    /// # Panics
    ///
    /// Panics if the token stream is empty or does not end with EOF. Use
    /// [`Parser::try_new`] when tokens may come from an untrusted source.
    #[must_use]
    pub fn new(tokens: Vec<Token>) -> Self {
        assert!(
            matches!(tokens.last().map(Token::kind), Some(TokenKind::Eof)),
            "token stream must end with an EOF token",
        );

        Self { tokens, current: 0 }
    }

    /// Creates a parser from a token stream, returning an error for an invalid stream.
    ///
    /// The stream must end with an EOF token. This is the non-panicking
    /// alternative to [`Parser::new`].
    ///
    /// # Errors
    ///
    /// Returns an error if the token stream is empty or does not end with EOF.
    pub fn try_new(tokens: Vec<Token>) -> Result<Self> {
        if matches!(tokens.last().map(Token::kind), Some(TokenKind::Eof)) {
            Ok(Self { tokens, current: 0 })
        } else {
            Err(Error::invalid_value("Token stream must end with an EOF token"))
        }
    }

    /// Returns a reference to the current token without consuming it.
    fn peek(&self) -> &Token { &self.tokens[self.current] }

    /// Advances the parser and returns the previously current token.
    ///
    /// The EOF token is never consumed.
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        &self.tokens[self.current.saturating_sub(1)]
    }

    /// Returns whether the current token is EOF.
    fn is_at_end(&self) -> bool { self.check(&TokenKind::Eof) }

    /// Checks whether the current token has the expected token kind.
    ///
    /// The comparison is based on the token variant, not its payload.
    fn check(&self, expected: &TokenKind) -> bool {
        std::mem::discriminant(self.peek().kind())
            == std::mem::discriminant(expected)
    }

    /// Consumes the current token if it matches the expected kind.
    ///
    /// # Errors
    ///
    /// Returns a positioned syntax error if the token does not match.
    fn consume(&mut self, expected: &TokenKind, message: &str) -> Result<()> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.syntax_error(message))
        }
    }

    /// Creates a syntax error at the current token position.
    ///
    /// EOF errors are classified as [`ErrorKind::UnexpectedEof`].
    fn syntax_error(&self, message: &str) -> Error {
        let error = if self.is_at_end() {
            Error::new(ErrorKind::UnexpectedEof, message)
        } else {
            Error::syntax(message)
        };

        error.with_position(self.peek().position())
    }

    /// Parses a numeric token into a [`Value`].
    ///
    /// Integer values use the smallest supported integer type
    /// that can represent the parsed value.
    ///
    /// Decimal and scientific notation values use [`Value::F64`].
    /// Floating-point values must be finite.
    ///
    /// # Errors
    ///
    /// Returns an error if the number is invalid, exceeds the
    /// supported integer range, or produces a non-finite value.
    fn parse_number(number: &str) -> Result<Value> {
        let result = if number.contains(['.', 'e', 'E']) {
            number
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .map(Value::F64)
        } else if number.starts_with('-') {
            number
                .parse::<i64>()
                .map(Value::I64)
                .or_else(|_| number.parse::<i128>().map(Value::I128))
                .ok()
        } else {
            number
                .parse::<i64>()
                .map(Value::I64)
                .or_else(|_| number.parse::<u64>().map(Value::U64))
                .or_else(|_| number.parse::<i128>().map(Value::I128))
                .or_else(|_| number.parse::<u128>().map(Value::U128))
                .ok()
        };

        result.ok_or_else(|| {
            Error::invalid_value(format!(
                "Invalid or unsupported number: {number}"
            ))
        })
    }

    /// Parses an object enclosed in parentheses.
    ///
    /// Duplicate property names replace previous values.
    ///
    /// # Errors
    ///
    /// Returns an error if a property is malformed, a separator
    /// is missing, or the closing parenthesis is absent.
    /// Parses an object enclosed in parentheses.
    ///
    /// Duplicate property names replace previous values.
    /// Properties are separated by commas or newlines.
    fn parse_object(&mut self) -> Result<Value> {
        self.consume(&TokenKind::LeftParen, "Expected '(' to start an object")?;

        let mut object = Value::empty_object();

        self.skip_newlines();

        while !self.check(&TokenKind::RightParen) {
            if self.is_at_end() {
                return Err(self.syntax_error("Unterminated object"));
            }

            let (key, value) = self.parse_property()?;

            if let Err(value) = object.insert(key, value) {
                return Err(self.syntax_error(&format!(
                    "Expected an object, got {value:?}"
                )));
            }

            if self.check(&TokenKind::RightParen) {
                break;
            }

            if self.is_at_end() {
                return Err(self.syntax_error("Unterminated object"));
            }

            self.consume_property_separator()?;
        }

        self.consume(
            &TokenKind::RightParen,
            "Expected ')' after object properties",
        )?;

        Ok(object)
    }

    /// Consumes a separator between collection elements.
    ///
    /// A comma or one or more newlines can separate elements.
    /// A comma may optionally follow a newline separator.
    ///
    /// # Errors
    ///
    /// Returns a syntax error if no valid separator is present.
    fn consume_collection_separator(&mut self) -> Result<()> {
        if self.check(&TokenKind::Comma) {
            self.advance();
            self.skip_newlines();
            return Ok(());
        }

        if self.check(&TokenKind::Newline) {
            self.skip_newlines();

            if self.check(&TokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            }

            return Ok(());
        }

        Err(self.syntax_error(
            "Expected ',' or newline between collection elements",
        ))
    }

    /// Parses an array enclosed in square brackets.
    ///
    /// Elements are separated by commas or newlines.
    /// Trailing commas are allowed.
    fn parse_array(&mut self) -> Result<Value> {
        self.consume(
            &TokenKind::LeftBracket,
            "Expected '[' to start an array",
        )?;

        let mut array = Vec::new();

        self.skip_newlines();

        while !self.check(&TokenKind::RightBracket) {
            if self.is_at_end() {
                return Err(self.syntax_error("Unterminated array"));
            }

            array.push(self.parse_value()?);

            if self.check(&TokenKind::RightBracket) {
                break;
            }

            if self.is_at_end() {
                return Err(self.syntax_error("Unterminated array"));
            }

            self.consume_collection_separator()?;
        }

        self.consume(
            &TokenKind::RightBracket,
            "Expected ']' after array elements",
        )?;

        Ok(Value::Array(array))
    }

    /// Parses a set enclosed in curly braces.
    ///
    /// Duplicate elements are automatically discarded.
    /// Elements are separated by commas or newlines.
    /// Trailing commas are allowed.
    fn parse_set(&mut self) -> Result<Value> {
        self.consume(&TokenKind::LeftBrace, "Expected '{' to start a set")?;

        let mut set = BTreeSet::new();

        self.skip_newlines();

        while !self.check(&TokenKind::RightBrace) {
            if self.is_at_end() {
                return Err(self.syntax_error("Unterminated set"));
            }

            set.insert(self.parse_value()?);

            if self.check(&TokenKind::RightBrace) {
                break;
            }

            if self.is_at_end() {
                return Err(self.syntax_error("Unterminated set"));
            }

            self.consume_collection_separator()?;
        }

        self.consume(
            &TokenKind::RightBrace,
            "Expected '}' after set elements",
        )?;

        Ok(Value::Set(set))
    }

    /// Consumes consecutive newline tokens.
    fn skip_newlines(&mut self) {
        while self.check(&TokenKind::Newline) {
            self.advance();
        }
    }

    /// Consumes a separator between object properties.
    ///
    /// A comma or one or more newlines can separate properties.
    /// A comma may optionally follow a newline separator.
    ///
    /// # Errors
    ///
    /// Returns a syntax error if no valid separator is present.
    fn consume_property_separator(&mut self) -> Result<()> {
        if self.check(&TokenKind::Comma) {
            self.advance();
            self.skip_newlines();
            return Ok(());
        }

        if self.check(&TokenKind::Newline) {
            self.skip_newlines();

            if self.check(&TokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            }

            return Ok(());
        }

        Err(self
            .syntax_error("Expected ',' or newline between object properties"))
    }

    /// Parses the value represented by the current token.
    ///
    /// Nested objects, arrays, and sets are parsed recursively.
    ///
    /// # Errors
    ///
    /// Returns an error if the current token cannot begin a valid value.
    fn parse_value(&mut self) -> Result<Value> {
        match self.peek().kind() {
            TokenKind::Null => {
                self.advance();
                Ok(Value::Null)
            }
            TokenKind::Bool(value) => {
                let value = *value;
                self.advance();
                Ok(Value::Bool(value))
            }
            TokenKind::String(value) => {
                let value = value.clone();
                self.advance();
                Ok(Value::String(value))
            }
            TokenKind::Number(number) => {
                let position = self.peek().position();
                let result = Self::parse_number(number);

                self.advance();

                result.map_err(|error| error.with_position(position))
            }
            TokenKind::LeftParen => self.parse_object(),
            TokenKind::LeftBracket => self.parse_array(),
            TokenKind::LeftBrace => self.parse_set(),
            _ => Err(self.syntax_error("Expected a value")),
        }
    }

    /// Parses a document property.
    ///
    /// A property consists of an identifier followed by an
    /// equals sign and a value.
    ///
    /// # Errors
    ///
    /// Returns an error if the identifier, assignment operator,
    /// or value is invalid.
    fn parse_property(&mut self) -> Result<(String, Value)> {
        let key = match self.peek().kind() {
            TokenKind::Identifier(key) => key.clone(),
            _ => {
                return Err(self.syntax_error("Expected a property identifier"));
            }
        };

        self.advance();

        self.consume(
            &TokenKind::Equals,
            "Expected '=' after property identifier",
        )?;

        let value = self.parse_value()?;

        Ok((key, value))
    }

    /// Parses the complete CBR document.
    ///
    /// A document can be represented as an object enclosed in
    /// parentheses or as a sequence of top-level properties.
    ///
    /// Duplicate property names replace previous values.
    /// Trailing commas are allowed.
    ///
    /// An empty document produces an empty object.
    ///
    /// # Errors
    ///
    /// Returns an error if the document contains invalid syntax.
    /// Parses the complete CBR document.
    ///
    /// The document root is represented by a sequence of
    /// properties without enclosing delimiters.
    ///
    /// Duplicate property names replace previous values.
    /// Trailing commas are allowed.
    ///
    /// An empty document produces an empty object.
    ///
    /// # Errors
    ///
    /// Returns an error if the document contains invalid syntax.
    /// Parses the complete CBR document.
    ///
    /// The document root is represented by a sequence of
    /// properties without enclosing delimiters.
    ///
    /// Properties are separated by commas or newlines.
    /// Duplicate property names replace previous values.
    ///
    /// An empty document produces an empty object.
    ///
    /// # Errors
    ///
    /// Returns an error if the document contains invalid syntax.
    pub fn parse(&mut self) -> Result<Value> {
        let mut object = Value::empty_object();

        self.skip_newlines();

        while !self.is_at_end() {
            let (key, value) = self.parse_property()?;

            if let Err(value) = object.insert(key, value) {
                return Err(self.syntax_error(&format!(
                    "Expected an object, got {value:?}"
                )));
            }

            if self.is_at_end() {
                break;
            }

            self.consume_property_separator()?;
        }

        Ok(object)
    }
}
