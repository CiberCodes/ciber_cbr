//! Integration tests for the [`Value`] type.
//!
//! This module verifies the behavior of CBR values, including
//! scalar variants, compound values, conversions, accessors,
//! object operations, equality, and ordering.
//!
//! The tests cover both successful operations and cases where
//! an accessor or operation is used with an incompatible value.
//!
//! # Test organization
//!
//! - Scalar value construction and conversions.
//! - Object creation and property operations.
//! - Array, byte sequence, and set accessors.
//! - Numeric and boolean accessors.
//! - Equality and total ordering.
//! - Default value behavior.

use std::collections::{BTreeMap, BTreeSet};

use ciber_cbr::value::Value;

// -----------------------------------------------------------------------------
// Construction and default
// -----------------------------------------------------------------------------

/// Verifies that an empty object can be created.
#[test]
fn empty_object() {
    let value = Value::empty_object();

    assert!(value.is_object());
    assert!(value.as_object().is_some_and(BTreeMap::is_empty));
}

/// Verifies that the default value is an empty object.
#[test]
fn default_value() {
    let value = Value::default();

    assert!(value.is_object());
    assert!(value.as_object().is_some_and(BTreeMap::is_empty));
}

/// Verifies that the null variant is recognized.
#[test]
fn null_value() {
    assert!(Value::Null.is_null());
    assert!(!Value::Bool(false).is_null());
}

/// Verifies that object variants are recognized.
#[test]
fn object_value() {
    assert!(Value::empty_object().is_object());
    assert!(!Value::Null.is_object());
}

// -----------------------------------------------------------------------------
// Object operations
// -----------------------------------------------------------------------------

/// Verifies insertion of a new object property.
#[test]
fn insert_new_property() {
    let mut value = Value::empty_object();

    let result = value.insert("name", Value::from("Cirious"));

    assert!(matches!(result, Ok(None)));
    assert_eq!(value.get("name"), Some(&Value::from("Cirious")));
}

/// Verifies replacement of an existing object property.
#[test]
fn insert_replaces_existing_property() {
    let mut value = Value::empty_object();

    assert!(matches!(
        value.insert("name", Value::from("CiberCodes")),
        Ok(None)
    ));

    let result = value.insert("name", Value::from("Cirious"));

    assert!(matches!(
      result,
      Ok(Some(Value::String(previous))) if previous == "CiberCodes"
    ));

    assert_eq!(value.get("name"), Some(&Value::from("Cirious")));
}

/// Verifies that insertion into a non-object returns the supplied value.
#[test]
fn insert_into_non_object() {
    let mut value = Value::Null;

    let result = value.insert("name", Value::Bool(true));

    assert!(matches!(result, Err(Value::Bool(true))));
    assert!(value.is_null());
}

/// Verifies retrieval of an existing object property.
#[test]
fn get_existing_property() {
    let mut value = Value::empty_object();

    assert!(matches!(value.insert("enabled", Value::Bool(true)), Ok(None)));

    assert_eq!(value.get("enabled"), Some(&Value::Bool(true)));
}

/// Verifies that retrieving a missing property returns `None`.
#[test]
fn get_missing_property() {
    let value = Value::empty_object();

    assert_eq!(value.get("missing"), None);
}

/// Verifies that retrieving a property from a non-object returns `None`.
#[test]
fn get_from_non_object() {
    assert_eq!(Value::Null.get("missing"), None);
}

/// Verifies mutable access to an existing object property.
#[test]
fn get_mut_existing_property() {
    let mut value = Value::empty_object();

    assert!(matches!(value.insert("enabled", Value::Bool(false)), Ok(None)));

    if let Some(enabled) = value.get_mut("enabled") {
        *enabled = Value::Bool(true);
    }

    assert_eq!(value.get("enabled"), Some(&Value::Bool(true)));
}

/// Verifies that mutable access to a missing property returns `None`.
#[test]
fn get_mut_missing_property() {
    let mut value = Value::empty_object();

    assert!(value.get_mut("missing").is_none());
}

/// Verifies that mutable access on a non-object returns `None`.
#[test]
fn get_mut_from_non_object() {
    let mut value = Value::Null;

    assert!(value.get_mut("missing").is_none());
    assert!(value.is_null());
}

/// Verifies the behavior of object key membership checks.
#[test]
fn contains_key() {
    let mut value = Value::empty_object();

    assert!(matches!(value.insert("name", Value::from("Cirious")), Ok(None)));

    assert!(value.contains_key("name"));
    assert!(!value.contains_key("missing"));
}

/// Verifies that key membership checks return false for non-objects.
#[test]
fn contains_key_on_non_object() {
    assert!(!Value::Null.contains_key("missing"));
}

/// Verifies removal of an existing object property.
#[test]
fn remove_existing_property() {
    let mut value = Value::empty_object();

    assert!(matches!(value.insert("name", Value::from("Cirious")), Ok(None)));

    assert_eq!(value.remove("name"), Some(Value::from("Cirious")));

    assert!(!value.contains_key("name"));
}

/// Verifies that removing a missing property returns `None`.
#[test]
fn remove_missing_property() {
    let mut value = Value::empty_object();

    assert_eq!(value.remove("missing"), None);
    assert!(value.is_object());
}

/// Verifies that removing a property from a non-object returns `None`.
#[test]
fn remove_from_non_object() {
    let mut value = Value::Null;

    assert_eq!(value.remove("missing"), None);
    assert!(value.is_null());
}

// -----------------------------------------------------------------------------
// Object accessors
// -----------------------------------------------------------------------------

/// Verifies immutable access to object entries.
#[test]
fn as_object() {
    let value = Value::empty_object();

    assert!(value.as_object().is_some());
    assert!(Value::Null.as_object().is_none());
}

/// Verifies mutable access to object entries.
#[test]
fn as_object_mut() {
    let mut value = Value::empty_object();

    if let Some(object) = value.as_object_mut() {
        object.insert("enabled".to_owned(), Value::Bool(true));
    }

    assert_eq!(value.get("enabled"), Some(&Value::Bool(true)));
}

/// Verifies that mutable object access rejects non-objects.
#[test]
fn as_object_mut_on_non_object() {
    let mut value = Value::Null;

    assert!(value.as_object_mut().is_none());
    assert!(value.is_null());
}

// -----------------------------------------------------------------------------
// Array accessors
// -----------------------------------------------------------------------------

/// Verifies immutable access to array elements.
#[test]
fn as_array() {
    let value = Value::Array(vec![Value::Bool(true), Value::I64(10)]);

    assert_eq!(value.as_array().map(<[_]>::len), Some(2));

    assert!(Value::Null.as_array().is_none());
}

/// Verifies mutable access to array elements.
#[test]
fn as_array_mut() {
    let mut value = Value::Array(vec![Value::Bool(true)]);

    if let Some(array) = value.as_array_mut() {
        array.push(Value::I64(10));
    }

    assert_eq!(value.as_array().map(<[_]>::len), Some(2));
}

/// Verifies that mutable array access rejects non-arrays.
#[test]
fn as_array_mut_on_non_array() {
    let mut value = Value::Null;

    assert!(value.as_array_mut().is_none());
}

// -----------------------------------------------------------------------------
// Byte accessors
// -----------------------------------------------------------------------------

/// Verifies immutable access to byte sequences.
#[test]
fn as_bytes() {
    let value = Value::Bytes(vec![1, 2, 3]);

    assert_eq!(value.as_bytes(), Some(&[1, 2, 3][..]));
    assert!(Value::Null.as_bytes().is_none());
}

/// Verifies mutable access to byte sequences.
#[test]
fn as_bytes_mut() {
    let mut value = Value::Bytes(vec![1, 2, 3]);

    if let Some(bytes) = value.as_bytes_mut() {
        bytes.push(4);
        bytes[0] = 9;
    }

    assert_eq!(value.as_bytes(), Some(&[9, 2, 3, 4][..]));
}

/// Verifies that mutable byte access rejects other variants.
#[test]
fn as_bytes_mut_on_non_bytes() {
    let mut value = Value::Null;

    assert!(value.as_bytes_mut().is_none());
}

// -----------------------------------------------------------------------------
// Set accessors
// -----------------------------------------------------------------------------

/// Verifies immutable access to sets.
#[test]
fn as_set() {
    let value = Value::Set(BTreeSet::from([Value::I64(1), Value::I64(2)]));

    assert_eq!(value.as_set().map(BTreeSet::len), Some(2));
    assert!(Value::Null.as_set().is_none());
}

/// Verifies mutable access to sets.
#[test]
fn as_set_mut() {
    let mut value = Value::Set(BTreeSet::new());

    if let Some(set) = value.as_set_mut() {
        set.insert(Value::I64(1));
        set.insert(Value::I64(2));
    }

    assert_eq!(value.as_set().map(BTreeSet::len), Some(2));
}

/// Verifies that sets automatically discard duplicate values.
#[test]
fn set_discards_duplicates() {
    let mut set = BTreeSet::new();

    set.insert(Value::I64(1));
    set.insert(Value::I64(1));

    let value = Value::Set(set);

    assert_eq!(value.as_set().map(BTreeSet::len), Some(1));
}

/// Verifies that mutable set access rejects other variants.
#[test]
fn as_set_mut_on_non_set() {
    let mut value = Value::Null;

    assert!(value.as_set_mut().is_none());
}

// -----------------------------------------------------------------------------
// Scalar accessors
// -----------------------------------------------------------------------------

/// Verifies immutable string access.
#[test]
fn as_str() {
    let value = Value::from("Cirious");

    assert_eq!(value.as_str(), Some("Cirious"));
    assert_eq!(Value::Null.as_str(), None);
}

/// Verifies boolean access.
#[test]
fn as_bool() {
    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert_eq!(Value::Bool(false).as_bool(), Some(false));
    assert_eq!(Value::Null.as_bool(), None);
}

/// Verifies signed integer access and range checks.
#[test]
fn as_i64() {
    assert_eq!(Value::I64(-10).as_i64(), Some(-10));
    assert_eq!(Value::I128(10).as_i64(), Some(10));
    assert_eq!(Value::U64(10).as_i64(), Some(10));
    assert_eq!(Value::U128(10).as_i64(), Some(10));

    assert_eq!(Value::I128(i128::MAX).as_i64(), None);
    assert_eq!(Value::U64(u64::MAX).as_i64(), None);
    assert_eq!(Value::U128(u128::MAX).as_i64(), None);
    assert_eq!(Value::F64(10.0).as_i64(), None);
    assert_eq!(Value::Null.as_i64(), None);
}

// -----------------------------------------------------------------------------
// Conversions
// -----------------------------------------------------------------------------

/// Verifies conversion from a boolean.
#[test]
fn from_bool() {
    assert!(matches!(Value::from(true), Value::Bool(true)));
}

/// Verifies conversion from a signed 64-bit integer.
#[test]
fn from_i64() {
    assert!(matches!(Value::from(-10_i64), Value::I64(-10)));
}

/// Verifies conversion from an unsigned 64-bit integer.
#[test]
fn from_u64() {
    assert!(matches!(Value::from(10_u64), Value::U64(10)));
}

/// Verifies conversion from a signed 128-bit integer.
#[test]
fn from_i128() {
    assert!(matches!(
      Value::from(i128::MAX),
      Value::I128(value) if value == i128::MAX
    ));
}

/// Verifies conversion from an unsigned 128-bit integer.
#[test]
fn from_u128() {
    assert!(matches!(
      Value::from(u128::MAX),
      Value::U128(value) if value == u128::MAX
    ));
}

/// Verifies conversion from a floating-point number.
#[test]
fn from_f64() {
    assert!(matches!(
      Value::from(2.5_f64),
      Value::F64(value) if value.total_cmp(&2.5).is_eq()
    ));
}

/// Verifies conversion from an owned string.
#[test]
fn from_string() {
    let value = Value::from(String::from("Cirious"));

    assert_eq!(value.as_str(), Some("Cirious"));
}

/// Verifies conversion from a string slice.
#[test]
fn from_str() {
    let value = Value::from("Cirious");

    assert_eq!(value.as_str(), Some("Cirious"));
}

/// Verifies conversion from a vector into an array.
#[test]
fn from_vec() {
    let value = Value::from(vec![1_i64, 2_i64, 3_i64]);

    assert_eq!(
        value.as_array(),
        Some(&[Value::I64(1), Value::I64(2), Value::I64(3),][..])
    );
}

// -----------------------------------------------------------------------------
// Equality and ordering
// -----------------------------------------------------------------------------

/// Verifies equality between identical values.
#[test]
fn equality() {
    assert_eq!(Value::I64(10), Value::I64(10));
    assert_ne!(Value::I64(10), Value::I64(20));
    assert_ne!(Value::I64(10), Value::U64(10));
}

/// Verifies total ordering between different variants.
#[test]
fn variant_ordering() {
    assert!(Value::Null < Value::Bool(false));
    assert!(Value::Bool(true) < Value::I64(0));
    assert!(Value::I64(0) < Value::U64(0));
    assert!(Value::U64(0) < Value::F64(0.0));
    assert!(Value::F64(0.0) < Value::I128(0));
    assert!(Value::I128(0) < Value::U128(0));
    assert!(Value::U128(0) < Value::from(""));
    assert!(Value::from("") < Value::Bytes(Vec::new()));
    assert!(Value::Bytes(Vec::new()) < Value::Array(Vec::new()));
    assert!(Value::Array(Vec::new()) < Value::empty_object());
    assert!(Value::empty_object() < Value::Set(BTreeSet::new()));
}

/// Verifies ordering within the same variant.
#[test]
fn same_variant_ordering() {
    assert!(Value::I64(1) < Value::I64(2));
    assert!(Value::from("a") < Value::from("b"));
    assert!(
        Value::Array(vec![Value::I64(1)]) < Value::Array(vec![Value::I64(2)])
    );
}

/// Verifies that floating-point ordering uses `total_cmp`.
#[test]
fn float_total_ordering() {
    assert_ne!(Value::F64(0.0), Value::F64(-0.0));

    assert!(Value::F64(-0.0) < Value::F64(0.0));

    assert_eq!(Value::F64(f64::NAN), Value::F64(f64::NAN));
}

/// Verifies that object equality and ordering use their entries.
#[test]
fn object_equality_and_ordering() {
    let first = Value::Object(BTreeMap::from([
        ("a".to_owned(), Value::I64(1)),
        ("b".to_owned(), Value::I64(2)),
    ]));

    let second = Value::Object(BTreeMap::from([
        ("a".to_owned(), Value::I64(1)),
        ("b".to_owned(), Value::I64(2)),
    ]));

    let third = Value::Object(BTreeMap::from([
        ("a".to_owned(), Value::I64(1)),
        ("b".to_owned(), Value::I64(3)),
    ]));

    assert_eq!(first, second);
    assert!(first < third);
}

/// Verifies that set equality and ordering use their elements.
#[test]
fn set_equality_and_ordering() {
    let first = Value::Set(BTreeSet::from([Value::I64(1), Value::I64(2)]));

    let second = Value::Set(BTreeSet::from([Value::I64(1), Value::I64(2)]));

    let third = Value::Set(BTreeSet::from([Value::I64(1), Value::I64(3)]));

    assert_eq!(first, second);
    assert!(first < third);
}

#[test]
fn as_u64_converts_valid_integer_variants() {
    assert_eq!(Value::I64(42).as_u64(), Some(42));
    assert_eq!(Value::U64(42).as_u64(), Some(42));
    assert_eq!(Value::I128(42).as_u64(), Some(42));
    assert_eq!(Value::U128(42).as_u64(), Some(42));
}

#[test]
fn as_u64_rejects_negative_and_out_of_range_values() {
    assert_eq!(Value::I64(-1).as_u64(), None);
    assert_eq!(Value::I128(-1).as_u64(), None);
    assert_eq!(Value::U128(u128::MAX).as_u64(), None);
}

#[test]
fn as_i128_converts_valid_integer_variants() {
    assert_eq!(Value::I64(42).as_i128(), Some(42));
    assert_eq!(Value::U64(42).as_i128(), Some(42));
    assert_eq!(Value::I128(42).as_i128(), Some(42));
    assert_eq!(Value::U128(42).as_i128(), Some(42));
}

#[test]
fn as_i128_rejects_out_of_range_values() {
    assert_eq!(Value::U128(u128::MAX).as_i128(), None);
}

#[test]
fn as_u128_converts_valid_integer_variants() {
    assert_eq!(Value::I64(42).as_u128(), Some(42));
    assert_eq!(Value::U64(42).as_u128(), Some(42));
    assert_eq!(Value::I128(42).as_u128(), Some(42));
    assert_eq!(Value::U128(42).as_u128(), Some(42));
}

#[test]
fn as_u128_rejects_negative_values() {
    assert_eq!(Value::I64(-1).as_u128(), None);
    assert_eq!(Value::I128(-1).as_u128(), None);
}

#[test]
fn as_f64_converts_numeric_variants() {
    assert_eq!(Value::I64(42).as_f64(), Some(42.0));
    assert_eq!(Value::U64(42).as_f64(), Some(42.0));
    assert_eq!(Value::I128(42).as_f64(), Some(42.0));
    assert_eq!(Value::U128(42).as_f64(), Some(42.0));
    assert_eq!(Value::F64(42.5).as_f64(), Some(42.5));
}

#[test]
fn numeric_accessors_reject_non_numeric_variants() {
    let value = Value::String("42".to_owned());

    assert_eq!(value.as_u64(), None);
    assert_eq!(value.as_i128(), None);
    assert_eq!(value.as_u128(), None);
    assert_eq!(value.as_f64(), None);
}

#[test]
fn as_u64_handles_numeric_boundaries() {
    assert_eq!(Value::I64(i64::MAX).as_u64(), Some(i64::MAX as u64));
    assert_eq!(Value::I128(i128::from(u64::MAX)).as_u64(), Some(u64::MAX));
    assert_eq!(Value::U128(u128::from(u64::MAX)).as_u64(), Some(u64::MAX));
    assert_eq!(Value::U128(u128::from(u64::MAX) + 1).as_u64(), None);
}

#[test]
fn as_i128_handles_numeric_boundaries() {
    assert_eq!(Value::I128(i128::MIN).as_i128(), Some(i128::MIN));
    assert_eq!(Value::I128(i128::MAX).as_i128(), Some(i128::MAX));
    assert_eq!(Value::U128(i128::MAX as u128).as_i128(), Some(i128::MAX));
    assert_eq!(Value::U128(i128::MAX as u128 + 1).as_i128(), None);
}

#[test]
fn as_u128_handles_numeric_boundaries() {
    assert_eq!(Value::I128(0).as_u128(), Some(0));
    assert_eq!(Value::I128(-1).as_u128(), None);
    assert_eq!(Value::U128(u128::MAX).as_u128(), Some(u128::MAX));
}

#[test]
fn as_f64_handles_special_values() {
    if let Some(value) = Value::F64(f64::NAN).as_f64() {
        assert!(value.is_nan());
    }

    assert_eq!(Value::F64(f64::INFINITY).as_f64(), Some(f64::INFINITY));
    assert_eq!(Value::F64(f64::NEG_INFINITY).as_f64(), Some(f64::NEG_INFINITY));
    assert_eq!(Value::F64(-0.0).as_f64(), Some(-0.0));
}
