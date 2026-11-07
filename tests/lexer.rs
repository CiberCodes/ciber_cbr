//! Integration tests for the CBR lexer.
//!
//! These tests verify tokenization of primitive values, identifiers,
//! strings, numbers, delimiters, and comments. They also cover source
//! positions, end-of-file handling, and invalid input.

use std::error::Error as StdError;

use ciber_cbr::lexer::{Token, TokenKind, tokenize};

/// Tokenizes the source and returns its token kinds.
fn token_kinds(source: &str) -> Result<Vec<TokenKind>, Box<dyn StdError>> {
  Ok(tokenize(source)?.into_iter().map(|token| token.kind().clone()).collect())
}

#[test]
fn tokenizes_empty_input() -> Result<(), Box<dyn StdError>> {
  assert_eq!(token_kinds("")?, vec![TokenKind::Eof],);

  Ok(())
}

#[test]
fn tokenizes_primitive_keywords() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds("true false null")?,
    vec![
      TokenKind::Bool(true),
      TokenKind::Bool(false),
      TokenKind::Null,
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn tokenizes_identifiers() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds("name user_name item-1 _private")?,
    vec![
      TokenKind::Identifier("name".to_owned()),
      TokenKind::Identifier("user_name".to_owned()),
      TokenKind::Identifier("item-1".to_owned()),
      TokenKind::Identifier("_private".to_owned()),
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn tokenizes_unicode_identifiers() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds("ação usuário configuração")?,
    vec![
      TokenKind::Identifier("ação".to_owned()),
      TokenKind::Identifier("usuário".to_owned()),
      TokenKind::Identifier("configuração".to_owned()),
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn tokenizes_strings() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds(r#""hello" "CBR" "" "#)?,
    vec![
      TokenKind::String("hello".to_owned()),
      TokenKind::String("CBR".to_owned()),
      TokenKind::String(String::new()),
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn tokenizes_string_escape_sequences() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds(
      r#""quote = \"" "slash = \\" "line\nbreak" "tab\t" "return\r" "null\0""#
    )?,
    vec![
      TokenKind::String("quote = \"".to_owned()),
      TokenKind::String("slash = \\".to_owned()),
      TokenKind::String("line\nbreak".to_owned()),
      TokenKind::String("tab\t".to_owned()),
      TokenKind::String("return\r".to_owned()),
      TokenKind::String("null\0".to_owned()),
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn tokenizes_integer_numbers() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds("0 42 -10 9223372036854775808")?,
    vec![
      TokenKind::Number("0".to_owned()),
      TokenKind::Number("42".to_owned()),
      TokenKind::Number("-10".to_owned()),
      TokenKind::Number("9223372036854775808".to_owned()),
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn tokenizes_floating_point_numbers() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds("3.14 -0.5 1e10 2.5E-3")?,
    vec![
      TokenKind::Number("3.14".to_owned()),
      TokenKind::Number("-0.5".to_owned()),
      TokenKind::Number("1e10".to_owned()),
      TokenKind::Number("2.5E-3".to_owned()),
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn tokenizes_delimiters_and_separators() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds("{}[]()==,")?,
    vec![
      TokenKind::LeftBrace,
      TokenKind::RightBrace,
      TokenKind::LeftBracket,
      TokenKind::RightBracket,
      TokenKind::LeftParen,
      TokenKind::RightParen,
      TokenKind::Equals,
      TokenKind::Equals,
      TokenKind::Comma,
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn tokenizes_object_property_syntax() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds("name = \"Ciber\", enabled = true")?,
    vec![
      TokenKind::Identifier("name".to_owned()),
      TokenKind::Equals,
      TokenKind::String("Ciber".to_owned()),
      TokenKind::Comma,
      TokenKind::Identifier("enabled".to_owned()),
      TokenKind::Equals,
      TokenKind::Bool(true),
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn ignores_line_comments() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds("name = \"Ciber\" // this is a comment\nenabled = true")?,
    vec![
      TokenKind::Identifier("name".to_owned()),
      TokenKind::Equals,
      TokenKind::String("Ciber".to_owned()),
      TokenKind::Newline,
      TokenKind::Identifier("enabled".to_owned()),
      TokenKind::Equals,
      TokenKind::Bool(true),
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn ignores_block_comments() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds("name = /* comment */ \"Ciber\" enabled = true")?,
    vec![
      TokenKind::Identifier("name".to_owned()),
      TokenKind::Equals,
      TokenKind::String("Ciber".to_owned()),
      TokenKind::Identifier("enabled".to_owned()),
      TokenKind::Equals,
      TokenKind::Bool(true),
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn ignores_multiline_block_comments() -> Result<(), Box<dyn StdError>> {
  assert_eq!(
    token_kinds("name = /* first line\nsecond line */ \"Ciber\"")?,
    vec![
      TokenKind::Identifier("name".to_owned()),
      TokenKind::Equals,
      TokenKind::String("Ciber".to_owned()),
      TokenKind::Eof,
    ],
  );

  Ok(())
}

#[test]
fn tracks_token_positions() -> Result<(), Box<dyn StdError>> {
  let tokens = tokenize("name = 1\nnext = true")?;

  let expected =
    [(1, 1), (1, 6), (1, 8), (1, 9), (2, 1), (2, 6), (2, 8), (2, 12)];

  assert_eq!(tokens.len(), expected.len());

  for (token, (line, column)) in tokens.iter().zip(expected) {
    assert_eq!(token.position().line(), line);
    assert_eq!(token.position().column(), column);
  }

  Ok(())
}

#[test]
fn tracks_positions_after_multiline_comments() -> Result<(), Box<dyn StdError>>
{
  let tokens = tokenize("/* first line\nsecond line */ name")?;

  assert_eq!(tokens[0].kind(), &TokenKind::Identifier("name".to_owned()));
  assert_eq!(tokens[0].position().line(), 2);
  assert_eq!(tokens[0].position().column(), 16);

  Ok(())
}

#[test]
fn appends_end_of_file_token() -> Result<(), Box<dyn StdError>> {
  let tokens = tokenize("name")?;

  assert_eq!(tokens.last().map(Token::kind), Some(&TokenKind::Eof),);

  Ok(())
}

#[test]
fn rejects_unterminated_strings() {
  assert!(tokenize(r#""unterminated"#).is_err());
}

#[test]
fn rejects_newlines_inside_strings() {
  assert!(tokenize("\"first line\nsecond line\"").is_err());
}

#[test]
fn rejects_unterminated_block_comments() {
  assert!(tokenize("/* unterminated comment").is_err());
}

#[test]
fn rejects_unexpected_characters() {
  assert!(tokenize("name = @").is_err());
}
