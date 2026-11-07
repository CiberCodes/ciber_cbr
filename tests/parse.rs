//! Test Cbr Parser

use std::str::FromStr;

use ciber_cbr::{CbrFile, Value, lexer::tokenize, parser::Parser};

#[test]
fn parses_empty_document() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = tokenize("")?;
    let mut parser = Parser::new(tokens);
    let value = parser.parse()?;

    assert!(matches!(
        value,
        Value::Object(object) if object.is_empty()
    ));

    Ok(())
}

#[test]
fn parses_top_level_properties() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = tokenize(
        r#"
        name = "CiberApp",
        version = 1,
        enabled = true,
        "#,
    )?;

    let mut parser = Parser::new(tokens);
    let value = parser.parse()?;

    let Value::Object(object) = value else {
        panic!("Expected an object");
    };

    assert_eq!(object.get("name"), Some(&Value::String("CiberApp".to_owned())));

    assert_eq!(object.get("version"), Some(&Value::I64(1)));

    assert_eq!(object.get("enabled"), Some(&Value::Bool(true)));

    Ok(())
}

#[test]
fn parses_nested_objects() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = tokenize(
        r#"
        application = (
            name = "CiberApp",
            database = (
                host = "localhost",
                port = 5432,
            ),
        )
        "#,
    )?;

    let mut parser = Parser::new(tokens);
    let value = parser.parse()?;

    let Value::Object(root) = value else {
        panic!("Expected an object");
    };

    assert!(matches!(root.get("application"), Some(Value::Object(_))));

    Ok(())
}

#[test]
fn parses_arrays() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = tokenize(r#"tags = ["rust", "serde", "cbr"]"#)?;

    let mut parser = Parser::new(tokens);
    let value = parser.parse()?;

    let Value::Object(object) = value else {
        panic!("Expected an object");
    };

    assert!(matches!(
        object.get("tags"),
        Some(Value::Array(values)) if values.len() == 3
    ));

    Ok(())
}

#[test]
fn parses_sets() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = tokenize(
        r"
        numbers = {1, 2, 3, 2, 1}
        ",
    )?;

    let mut parser = Parser::new(tokens);
    let value = parser.parse()?;

    let Value::Object(object) = value else {
        panic!("Expected an object");
    };

    assert!(matches!(
        object.get("numbers"),
        Some(Value::Set(values)) if values.len() == 3
    ));

    Ok(())
}

fn assert_parse_error(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);

    let Err(error) = parser.parse() else {
        return Err(format!("Expected parsing to fail for = {source:?}").into());
    };

    assert!(
        error.position().is_some(),
        "Expected an error position for = {source:?}"
    );

    Ok(())
}

#[test]
fn rejects_missing_property_separator() -> Result<(), Box<dyn std::error::Error>>
{
    assert_parse_error(r#"name "CiberApp""#)?;
    Ok(())
}

#[test]
fn rejects_missing_property_value() -> Result<(), Box<dyn std::error::Error>> {
    assert_parse_error("name = ")?;
    Ok(())
}

#[test]
fn accepts_properties_separated_by_newlines()
-> Result<(), Box<dyn std::error::Error>> {
    let file = CbrFile::from_str(
        r#"
        name = "CiberApp"
        version = 1
        "#,
    )?;

    assert_eq!(
        file.value().get("name").and_then(Value::as_str),
        Some("CiberApp")
    );
    assert_eq!(file.value().get("version").and_then(Value::as_i64), Some(1));

    Ok(())
}

#[test]
fn rejects_unclosed_object() -> Result<(), Box<dyn std::error::Error>> {
    assert_parse_error(
        r#"
        application = (
            name = "CiberApp"
        "#,
    )?;
    Ok(())
}

#[test]
fn rejects_unclosed_array() -> Result<(), Box<dyn std::error::Error>> {
    assert_parse_error(
        r#"
        tags = ["rust", "serde"
        "#,
    )?;
    Ok(())
}

#[test]
fn rejects_unclosed_set() -> Result<(), Box<dyn std::error::Error>> {
    assert_parse_error(
        r#"
        servers = {
            (host = "localhost", port = 8080)
        "#,
    )?;
    Ok(())
}

#[test]
fn parses_integer_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("0", Value::I64(0)),
        ("42", Value::I64(42)),
        ("-1", Value::I64(-1)),
        ("9223372036854775807", Value::I64(i64::MAX)),
        ("-9223372036854775808", Value::I64(i64::MIN)),
        ("9223372036854775808", Value::U64(9_223_372_036_854_775_808)),
        ("18446744073709551615", Value::U64(u64::MAX)),
        ("-170141183460469231731687303715884105728", Value::I128(i128::MIN)),
        ("170141183460469231731687303715884105727", Value::I128(i128::MAX)),
        ("340282366920938463463374607431768211455", Value::U128(u128::MAX)),
    ];

    for (source, expected) in cases {
        let tokens = tokenize(&format!("value = {source}"))?;

        let mut parser = Parser::new(tokens);
        let parsed = parser.parse()?;

        let Value::Object(object) = parsed else {
            return Err("Expected an object".into());
        };

        assert_eq!(object.get("value"), Some(&expected));
    }

    Ok(())
}

#[test]
#[allow(clippy::approx_constant)]
fn parses_floating_point_numbers() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("3.14", 3.14_f64),
        ("-0.5", -0.5_f64),
        ("0.0", 0.0_f64),
        ("1e3", 1_000.0_f64),
        ("2.5e-4", 0.00025_f64),
    ];

    for (source, expected) in cases {
        let tokens = tokenize(&format!("value = {source}"))?;
        let mut parser = Parser::new(tokens);
        let parsed = parser.parse()?;

        let Value::Object(object) = parsed else {
            return Err("Expected an object".into());
        };

        let Some(Value::F64(actual)) = object.get("value") else {
            return Err("Expected an F64 value".into());
        };

        assert_eq!(*actual, expected);
    }

    Ok(())
}

#[test]
fn rejects_numbers_outside_supported_ranges()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        "-170141183460469231731687303715884105729",
        "340282366920938463463374607431768211456",
        "1e9999",
        "-1e9999",
    ];

    for source in cases {
        let tokens = tokenize(&format!("value = {source}"))?;
        let mut parser = Parser::new(tokens);

        assert!(parser.parse().is_err(), "Expected {source} to be rejected");
    }

    Ok(())
}

#[test]
fn parses_values_above_i128_max() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = tokenize("value = 170141183460469231731687303715884105728")?;
    let mut parser = Parser::new(tokens);
    let parsed = parser.parse()?;

    let Value::Object(object) = parsed else {
        return Err("Expected an object".into());
    };

    assert_eq!(
        object.get("value"),
        Some(&Value::U128(170_141_183_460_469_231_731_687_303_715_884_105_728)),
    );

    Ok(())
}

#[test]
fn parses_parenthesized_root_object() -> Result<(), Box<dyn std::error::Error>>
{
    let tokens = tokenize("name = \"Cirious\", version = 1")?;
    let mut parser = Parser::new(tokens);

    let value = parser.parse()?;

    assert!(value.is_object());

    Ok(())
}

#[test]
fn rejects_trailing_tokens_after_root_object()
-> Result<(), Box<dyn std::error::Error>> {
    let tokens = tokenize("(name = \"Cirious\") extra")?;
    let mut parser = Parser::new(tokens);

    assert!(parser.parse().is_err());

    Ok(())
}
