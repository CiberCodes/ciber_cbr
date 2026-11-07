//! CBR formatting functionality.
//!
//! This module provides compact and pretty-print formatting for
//! [`Value`], as well as functions for writing formatted values
//! to an output writer.
//!
//! Objects and sets are formatted deterministically. The generated
//! output follows the syntax accepted by the CBR parser.
//!
//! Integer variants share the same numeric representation.
//! Parsing a formatted integer may produce a different integer
//! variant while preserving its numeric value.

use std::io::Write;

use crate::{
  error::{Error, ErrorKind, Result},
  value::Value,
};

/// The indentation width used by pretty-print formatting.
const INDENT_WIDTH: usize = 4;

/// Formats a value using compact CBR notation.
///
/// Nested values are kept on a single line, with a single space
/// following each comma.
///
/// Integer variants are formatted without type-specific suffixes.
/// Consequently, parsing the result may produce a different
/// integer variant while preserving its numeric value.
///
/// # Errors
///
/// Returns an error if the value cannot be represented using the
/// supported CBR syntax.
///
/// # Examples
///
/// ```
/// use ciber_cbr::{formatter, value::Value};
///
/// let value = Value::I64(42);
///
/// assert_eq!(formatter::to_string(&value)?, "42");
///
/// # Ok::<(), ciber_cbr::error::Error>(())
/// ```
pub fn to_string(value: &Value) -> Result<String> {
  format_to_string(value, false)
}

/// Formats a value using pretty-print CBR notation.
///
/// Nested objects, arrays, and sets are formatted across multiple
/// lines with four spaces of indentation per nesting level.
///
/// Empty collections remain on a single line.
///
/// # Errors
///
/// Returns an error if the value cannot be represented using the
/// supported CBR syntax.
///
/// # Examples
///
/// ```
/// use ciber_cbr::{formatter, value::Value};
///
/// let value = Value::Array(vec![Value::I64(1), Value::I64(2)]);
///
/// assert_eq!(formatter::to_string_pretty(&value)?, "[\n    1,\n    2\n]",);
///
/// # Ok::<(), ciber_cbr::error::Error>(())
/// ```
pub fn to_string_pretty(value: &Value) -> Result<String> {
  format_to_string(value, true)
}

/// Writes a value using compact CBR notation.
///
/// # Errors
///
/// Streams formatted fragments directly to the supplied writer without
/// allocating a complete intermediate document string.
///
/// Returns an error if formatting fails or if writing to the supplied writer fails.
///
/// # Examples
///
/// ```
/// use ciber_cbr::{formatter, value::Value};
///
/// let mut output = Vec::new();
///
/// formatter::to_writer(&Value::Bool(true), &mut output)?;
///
/// assert_eq!(output, b"true");
///
/// # Ok::<(), ciber_cbr::error::Error>(())
/// ```
pub fn to_writer<W>(value: &Value, writer: W) -> Result<()>
where
  W: Write,
{
  format_to_writer(value, writer, false)
}

/// Writes a value using pretty-print CBR notation.
///
/// # Errors
///
/// Streams pretty-formatted fragments directly to the supplied writer without
/// allocating a complete intermediate document string.
///
/// Returns an error if formatting fails or if writing to the supplied writer fails.
///
/// # Examples
///
/// ```
/// use ciber_cbr::{formatter, value::Value};
///
/// let mut output = Vec::new();
///
/// formatter::to_writer_pretty(&Value::I64(42), &mut output)?;
///
/// assert_eq!(output, b"42");
///
/// # Ok::<(), ciber_cbr::error::Error>(())
/// ```
pub fn to_writer_pretty<W>(value: &Value, writer: W) -> Result<()>
where
  W: Write,
{
  format_to_writer(value, writer, true)
}

fn format_to_string(value: &Value, pretty: bool) -> Result<String> {
  let mut bytes = Vec::new();
  format_to_writer(value, &mut bytes, pretty)?;
  String::from_utf8(bytes)
    .map_err(|_| formatting_error("formatter emitted invalid UTF-8"))
}

/// Emits formatting fragments directly to the destination.
fn format_to_writer<W: Write>(
  value: &Value,
  mut output: W,
  pretty: bool,
) -> Result<()> {
  match value {
    Value::Object(values) => {
      write_object_contents(values, &mut output, pretty, 0)?;
    }

    _ => {
      write_value(value, &mut output, pretty, 0)?;
    }
  }

  Ok(())
}

/// Writes a value into the output buffer.
fn write_value(
  value: &Value,
  output: &mut impl Write,
  pretty: bool,
  depth: usize,
) -> Result<()> {
  match value {
    Value::Null => output.write_all(b"null")?,

    Value::Bool(value) => {
      output.write_all(if *value { b"true" } else { b"false" })?;
    }

    Value::I64(value) => write!(output, "{value}")?,
    Value::U64(value) => write!(output, "{value}")?,
    Value::I128(value) => write!(output, "{value}")?,
    Value::U128(value) => write!(output, "{value}")?,

    Value::F64(value) => {
      write_float(*value, output)?;
    }

    Value::String(value) => {
      write_string(value, output)?;
    }

    Value::Bytes(_) => {
      return Err(formatting_error(
        "Byte values do not have a supported CBR representation",
      ));
    }

    Value::Array(values) => {
      write_array(values, output, pretty, depth)?;
    }

    Value::Object(values) => {
      write_object(values, output, pretty, depth)?;
    }

    Value::Set(values) => {
      write_set(values, output, pretty, depth)?;
    }
  }

  Ok(())
}

/// Writes a floating-point number.
///
/// Non-finite values are rejected because the CBR parser only
/// accepts finite floating-point numbers.
///
/// Integral floating-point values receive a decimal suffix
/// to preserve their floating-point interpretation.
fn write_float(value: f64, output: &mut impl Write) -> Result<()> {
  if !value.is_finite() {
    return Err(formatting_error(
      "Non-finite floating-point values are not supported",
    ));
  }

  let mut formatted = value.to_string();

  if !formatted.contains(['.', 'e', 'E']) {
    formatted.push_str(".0");
  }

  output.write_all(formatted.as_bytes())?;

  Ok(())
}

/// Writes a quoted and escaped string.
fn write_string(value: &str, output: &mut impl Write) -> Result<()> {
  output.write_all(b"\"")?;

  for character in value.chars() {
    match character {
      '"' => output.write_all(b"\\\"")?,
      '\\' => output.write_all(b"\\\\")?,
      '\n' => output.write_all(b"\\n")?,
      '\r' => output.write_all(b"\\r")?,
      '\t' => output.write_all(b"\\t")?,
      '\0' => output.write_all(b"\\0")?,

      character if character.is_control() => {
        return Err(formatting_error(
          "String contains a control character without a supported escape",
        ));
      }

      character => {
        let mut encoded = [0; 4];
        output.write_all(character.encode_utf8(&mut encoded).as_bytes())?;
      }
    }
  }

  output.write_all(b"\"")?;

  Ok(())
}

/// Writes an array.
fn write_array(
  values: &[Value],
  output: &mut impl Write,
  pretty: bool,
  depth: usize,
) -> Result<()> {
  output.write_all(b"[")?;

  if !values.is_empty() {
    if pretty {
      output.write_all(b"\n")?;
    }

    for (index, value) in values.iter().enumerate() {
      if pretty {
        write_indent(output, depth + 1)?;
      }

      write_value(value, output, pretty, depth + 1)?;

      if index + 1 < values.len() {
        output.write_all(b",")?;
      }

      if pretty {
        output.write_all(b"\n")?;
      } else if index + 1 < values.len() {
        output.write_all(b" ")?;
      }
    }

    if pretty {
      write_indent(output, depth)?;
    }
  }

  output.write_all(b"]")?;

  Ok(())
}

/// Writes an object without enclosing delimiters.
fn write_object_contents(
  values: &std::collections::BTreeMap<String, Value>,
  output: &mut impl Write,
  pretty: bool,
  depth: usize,
) -> Result<()> {
  if !values.is_empty() {
    if pretty {
      output.write_all(b"\n")?;
    }

    for (index, (key, value)) in values.iter().enumerate() {
      validate_identifier(key)?;

      if pretty {
        write_indent(output, depth + 1)?;
      }

      output.write_all(key.as_bytes())?;
      output.write_all(b" = ")?;

      write_value(value, output, pretty, depth + 1)?;

      if index + 1 < values.len() {
        output.write_all(b",")?;
      }

      if pretty {
        output.write_all(b"\n")?;
      } else if index + 1 < values.len() {
        output.write_all(b" ")?;
      }
    }

    if pretty {
      write_indent(output, depth)?;
    }
  }

  Ok(())
}

/// Writes a nested object enclosed in parentheses.
fn write_object(
  values: &std::collections::BTreeMap<String, Value>,
  output: &mut impl Write,
  pretty: bool,
  depth: usize,
) -> Result<()> {
  output.write_all(b"(")?;

  write_object_contents(values, output, pretty, depth)?;

  output.write_all(b")")?;

  Ok(())
}

/// Writes a set.
fn write_set(
  values: &std::collections::BTreeSet<Value>,
  output: &mut impl Write,
  pretty: bool,
  depth: usize,
) -> Result<()> {
  output.write_all(b"{")?;

  if !values.is_empty() {
    if pretty {
      output.write_all(b"\n")?;
    }

    for (index, value) in values.iter().enumerate() {
      if pretty {
        write_indent(output, depth + 1)?;
      }

      write_value(value, output, pretty, depth + 1)?;

      if index + 1 < values.len() {
        output.write_all(b",")?;
      }

      if pretty {
        output.write_all(b"\n")?;
      } else if index + 1 < values.len() {
        output.write_all(b" ")?;
      }
    }

    if pretty {
      write_indent(output, depth)?;
    }
  }

  output.write_all(b"}")?;

  Ok(())
}

/// Writes indentation for the specified nesting depth.
fn write_indent(output: &mut impl Write, depth: usize) -> Result<()> {
  const SPACES: &[u8; 64] =
    b"                                                                ";
  let mut remaining = depth.saturating_mul(INDENT_WIDTH);
  while remaining > 0 {
    let count = remaining.min(SPACES.len());
    output.write_all(&SPACES[..count])?;
    remaining -= count;
  }
  Ok(())
}

/// Validates an identifier according to the CBR lexer rules.
fn validate_identifier(identifier: &str) -> Result<()> {
  let mut characters = identifier.chars();

  let Some(first) = characters.next() else {
    return Err(formatting_error("Object keys cannot be empty"));
  };

  let valid_start = first.is_alphabetic() || first == '_';

  let valid_rest = characters.clone().all(|character| {
    character.is_alphanumeric() || character == '_' || character == '-'
  });

  let valid_ending = !identifier.ends_with('-');

  let reserved = matches!(identifier, "true" | "false" | "null");

  if !valid_start || !valid_rest || !valid_ending || reserved {
    return Err(formatting_error("Object key is not a valid CBR identifier"));
  }

  Ok(())
}

/// Creates a formatting error.
fn formatting_error(message: impl Into<String>) -> Error {
  Error::new(ErrorKind::Formatting, message)
}
