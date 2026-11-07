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
pub fn to_string(value: &Value) -> Result<String> { format_value(value, false) }

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
    format_value(value, true)
}

/// Writes a value using compact CBR notation.
///
/// # Errors
///
/// Returns an error if formatting fails or if writing to the
/// supplied writer fails.
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
pub fn to_writer<W>(value: &Value, mut writer: W) -> Result<()>
where
    W: Write,
{
    let output = to_string(value)?;
    writer.write_all(output.as_bytes())?;

    Ok(())
}

/// Writes a value using pretty-print CBR notation.
///
/// # Errors
///
/// Returns an error if formatting fails or if writing to the
/// supplied writer fails.
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
pub fn to_writer_pretty<W>(value: &Value, mut writer: W) -> Result<()>
where
    W: Write,
{
    let output = to_string_pretty(value)?;
    writer.write_all(output.as_bytes())?;

    Ok(())
}

/// Formats a value using the selected formatting mode.
fn format_value(value: &Value, pretty: bool) -> Result<String> {
    let mut output = String::new();

    match value {
        Value::Object(values) => {
            write_object_contents(values, &mut output, pretty, 0)?;
        }

        _ => {
            write_value(value, &mut output, pretty, 0)?;
        }
    }

    Ok(output)
}

/// Writes a value into the output buffer.
fn write_value(
    value: &Value,
    output: &mut String,
    pretty: bool,
    depth: usize,
) -> Result<()> {
    match value {
        Value::Null => output.push_str("null"),

        Value::Bool(value) => {
            output.push_str(if *value { "true" } else { "false" });
        }

        Value::I64(value) => output.push_str(&value.to_string()),
        Value::U64(value) => output.push_str(&value.to_string()),
        Value::I128(value) => output.push_str(&value.to_string()),
        Value::U128(value) => output.push_str(&value.to_string()),

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
fn write_float(value: f64, output: &mut String) -> Result<()> {
    if !value.is_finite() {
        return Err(formatting_error(
            "Non-finite floating-point values are not supported",
        ));
    }

    let mut formatted = value.to_string();

    if !formatted.contains(['.', 'e', 'E']) {
        formatted.push_str(".0");
    }

    output.push_str(&formatted);

    Ok(())
}

/// Writes a quoted and escaped string.
fn write_string(value: &str, output: &mut String) -> Result<()> {
    output.push('"');

    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\0' => output.push_str("\\0"),

            character if character.is_control() => {
                return Err(formatting_error(
                    "String contains a control character without a supported escape",
                ));
            }

            character => output.push(character),
        }
    }

    output.push('"');

    Ok(())
}

/// Writes an array.
fn write_array(
    values: &[Value],
    output: &mut String,
    pretty: bool,
    depth: usize,
) -> Result<()> {
    output.push('[');

    if !values.is_empty() {
        if pretty {
            output.push('\n');
        }

        for (index, value) in values.iter().enumerate() {
            if pretty {
                write_indent(output, depth + 1);
            }

            write_value(value, output, pretty, depth + 1)?;

            if index + 1 < values.len() {
                output.push(',');
            }

            if pretty {
                output.push('\n');
            } else if index + 1 < values.len() {
                output.push(' ');
            }
        }

        if pretty {
            write_indent(output, depth);
        }
    }

    output.push(']');

    Ok(())
}

/// Writes an object without enclosing delimiters.
fn write_object_contents(
    values: &std::collections::BTreeMap<String, Value>,
    output: &mut String,
    pretty: bool,
    depth: usize,
) -> Result<()> {
    if !values.is_empty() {
        if pretty {
            output.push('\n');
        }

        for (index, (key, value)) in values.iter().enumerate() {
            validate_identifier(key)?;

            if pretty {
                write_indent(output, depth + 1);
            }

            output.push_str(key);
            output.push_str(" = ");

            write_value(value, output, pretty, depth + 1)?;

            if index + 1 < values.len() {
                output.push(',');
            }

            if pretty {
                output.push('\n');
            } else if index + 1 < values.len() {
                output.push(' ');
            }
        }

        if pretty {
            write_indent(output, depth);
        }
    }

    Ok(())
}

/// Writes a nested object enclosed in parentheses.
fn write_object(
    values: &std::collections::BTreeMap<String, Value>,
    output: &mut String,
    pretty: bool,
    depth: usize,
) -> Result<()> {
    output.push('(');

    write_object_contents(values, output, pretty, depth)?;

    output.push(')');

    Ok(())
}

/// Writes a set.
fn write_set(
    values: &std::collections::BTreeSet<Value>,
    output: &mut String,
    pretty: bool,
    depth: usize,
) -> Result<()> {
    output.push('{');

    if !values.is_empty() {
        if pretty {
            output.push('\n');
        }

        for (index, value) in values.iter().enumerate() {
            if pretty {
                write_indent(output, depth + 1);
            }

            write_value(value, output, pretty, depth + 1)?;

            if index + 1 < values.len() {
                output.push(',');
            }

            if pretty {
                output.push('\n');
            } else if index + 1 < values.len() {
                output.push(' ');
            }
        }

        if pretty {
            write_indent(output, depth);
        }
    }

    output.push('}');

    Ok(())
}

/// Writes indentation for the specified nesting depth.
fn write_indent(output: &mut String, depth: usize) {
    output.push_str(&" ".repeat(depth * INDENT_WIDTH));
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
        return Err(formatting_error(
            "Object key is not a valid CBR identifier",
        ));
    }

    Ok(())
}

/// Creates a formatting error.
fn formatting_error(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Formatting, message)
}
