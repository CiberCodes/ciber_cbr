//! Integration tests for the CBR formatter.
//!
//! These tests verify primitive values, collections, nested structures,
//! deterministic output, pretty formatting, and parser compatibility.

use std::collections::BTreeSet;

use ciber_cbr::{
    Result, formatter, lexer::tokenize, parser::Parser, value::Value,
};

/// Formats a value and compares the result with the expected output.
fn assert_formatted(value: &Value, expected: &str) -> Result<()> {
    let actual = formatter::to_string(value)?;

    assert_eq!(actual, expected);

    Ok(())
}

/// Inserts a property into an object without replacing an existing value.
fn insert_property(object: &mut Value, key: &str, value: Value) {
    let result = object.insert(key, value);

    assert!(
        matches!(result, Ok(None)),
        "Expected property insertion to succeed without replacing a value",
    );
}

#[test]
fn formats_null() -> Result<()> { assert_formatted(&Value::Null, "null") }

#[test]
fn formats_booleans() -> Result<()> {
    assert_formatted(&Value::Bool(true), "true")?;
    assert_formatted(&Value::Bool(false), "false")
}

#[test]
fn formats_signed_integers() -> Result<()> {
    assert_formatted(&Value::I64(42), "42")?;
    assert_formatted(&Value::I64(-42), "-42")?;
    assert_formatted(&Value::I128(i128::MIN), &i128::MIN.to_string())
}

#[test]
fn formats_unsigned_integers() -> Result<()> {
    assert_formatted(&Value::U64(u64::MAX), &u64::MAX.to_string())?;
    assert_formatted(&Value::U128(u128::MAX), &u128::MAX.to_string())
}

#[test]
fn formats_floating_point_numbers() -> Result<()> {
    assert_formatted(&Value::F64(3.5), "3.5")?;
    assert_formatted(&Value::F64(-0.5), "-0.5")?;
    assert_formatted(&Value::F64(42.0), "42.0")?;
    assert_formatted(&Value::F64(-0.0), "-0.0")
}

#[test]
fn rejects_non_finite_floating_point_numbers() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let result = formatter::to_string(&Value::F64(value));

        assert!(result.is_err());
    }
}

#[test]
fn formats_strings() -> Result<()> {
    assert_formatted(&Value::String("Ciber".to_owned()), "\"Ciber\"")
}

#[test]
fn formats_escaped_strings() -> Result<()> {
    assert_formatted(
        &Value::String("quote = \"".to_owned()),
        "\"quote = \\\"\"",
    )?;

    assert_formatted(
        &Value::String("path = \\".to_owned()),
        "\"path = \\\\\"",
    )?;

    assert_formatted(
        &Value::String("line\nbreak".to_owned()),
        "\"line\\nbreak\"",
    )?;

    assert_formatted(
        &Value::String("column\rreturn".to_owned()),
        "\"column\\rreturn\"",
    )?;

    assert_formatted(
        &Value::String("tab\tcharacter".to_owned()),
        "\"tab\\tcharacter\"",
    )?;

    assert_formatted(
        &Value::String("null\0character".to_owned()),
        "\"null\\0character\"",
    )
}

#[test]
fn rejects_unsupported_control_characters() {
    let value = Value::String("unsupported\u{0001}control".to_owned());

    assert!(formatter::to_string(&value).is_err());
}

#[test]
fn rejects_byte_values() {
    let value = Value::Bytes(vec![1, 2, 3]);

    assert!(formatter::to_string(&value).is_err());
}

#[test]
fn formats_empty_arrays() -> Result<()> {
    assert_formatted(&Value::Array(Vec::new()), "[]")
}

#[test]
fn formats_arrays() -> Result<()> {
    let value = Value::Array(vec![Value::I64(1), Value::I64(2), Value::I64(3)]);

    assert_formatted(&value, "[1, 2, 3]")
}

#[test]
fn formats_empty_objects() -> Result<()> {
    assert_formatted(&Value::empty_object(), "")
}

#[test]
fn formats_objects() -> Result<()> {
    let mut value = Value::empty_object();

    insert_property(&mut value, "name", Value::String("Ciber".to_owned()));

    insert_property(&mut value, "enabled", Value::Bool(true));

    assert_formatted(&value, "enabled = true, name = \"Ciber\"")
}

#[test]
fn formats_empty_sets() -> Result<()> {
    assert_formatted(&Value::Set(BTreeSet::default()), "{}")
}

#[test]
fn formats_sets_deterministically() -> Result<()> {
    let mut set = BTreeSet::new();

    set.insert(Value::I64(3));
    set.insert(Value::I64(1));
    set.insert(Value::I64(2));

    assert_formatted(&Value::Set(set), "{1, 2, 3}")
}

#[test]
fn formats_nested_values() -> Result<()> {
    let mut object = Value::empty_object();

    insert_property(
        &mut object,
        "items",
        Value::Array(vec![Value::I64(1), Value::I64(2)]),
    );

    assert_formatted(&object, "items = [1, 2]")
}

#[test]
fn formats_objects_deterministically() -> Result<()> {
    let mut first = Value::empty_object();

    insert_property(&mut first, "z", Value::I64(1));
    insert_property(&mut first, "a", Value::I64(2));

    let mut second = Value::empty_object();

    insert_property(&mut second, "a", Value::I64(2));
    insert_property(&mut second, "z", Value::I64(1));

    let first_output = formatter::to_string(&first)?;
    let second_output = formatter::to_string(&second)?;

    assert_eq!(first_output, second_output);

    Ok(())
}

#[test]
fn pretty_formats_arrays() -> Result<()> {
    let value = Value::Array(vec![Value::I64(1), Value::I64(2)]);

    assert_eq!(formatter::to_string_pretty(&value)?, "[\n    1,\n    2\n]",);

    Ok(())
}

#[test]
fn pretty_formats_empty_collections() -> Result<()> {
    assert_eq!(formatter::to_string_pretty(&Value::Array(Vec::new()))?, "[]",);

    assert_eq!(formatter::to_string_pretty(&Value::empty_object())?, "",);

    assert_eq!(
        formatter::to_string_pretty(&Value::Set(BTreeSet::new()))?,
        "{}",
    );

    Ok(())
}

#[test]
fn pretty_formats_objects() -> Result<()> {
    let mut object = Value::empty_object();

    insert_property(&mut object, "name", Value::String("Ciber".to_owned()));
    insert_property(&mut object, "enabled", Value::Bool(true));

    assert_eq!(
        formatter::to_string_pretty(&object)?,
        "\n    enabled = true,\n    name = \"Ciber\"\n",
    );

    Ok(())
}

#[test]
fn writes_compact_output() -> Result<()> {
    let mut output = Vec::new();

    formatter::to_writer(&Value::Bool(true), &mut output)?;

    assert_eq!(output, b"true");

    Ok(())
}

#[test]
fn writes_pretty_output() -> Result<()> {
    let mut output = Vec::new();

    formatter::to_writer_pretty(
        &Value::Array(vec![Value::I64(1), Value::I64(2)]),
        &mut output,
    )?;

    assert_eq!(output, b"[\n    1,\n    2\n]");

    Ok(())
}

#[test]
fn round_trips_integer_values() -> Result<()> {
    let values = [
        Value::I64(-42),
        Value::I64(42),
        Value::U64(42),
        Value::I128(42),
        Value::U128(42),
    ];

    for value in values {
        let mut document = Value::empty_object();

        insert_property(&mut document, "value", value.clone());

        let formatted = formatter::to_string(&document)?;
        let tokens = tokenize(&formatted)?;
        let mut parser = Parser::new(tokens);
        let parsed = parser.parse()?;

        assert_eq!(
            parsed.get("value"),
            Some(&match value {
                Value::I64(number) => Value::I64(number),
                Value::U64(number) => Value::I64(i64::try_from(number)?),
                Value::I128(number) => Value::I64(i64::try_from(number)?),
                Value::U128(number) => Value::I64(i64::try_from(number)?),
                _ => unreachable!(),
            })
        );
    }

    Ok(())
}

#[test]
fn round_trips_floating_point_values() -> Result<()> {
    let values =
        [Value::F64(3.5), Value::F64(-0.5), Value::F64(42.0), Value::F64(-0.0)];

    for value in values {
        let mut document = Value::empty_object();

        insert_property(&mut document, "value", value.clone());

        let formatted = formatter::to_string(&document)?;
        let tokens = tokenize(&formatted)?;
        let mut parser = Parser::new(tokens);
        let parsed = parser.parse()?;

        assert_eq!(parsed.get("value"), Some(&value));
    }

    Ok(())
}
