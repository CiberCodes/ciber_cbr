//! Values used by the Ciber CBR data format.
//!
//! This module defines [`Value`], the owned in-memory
//! representation of CBR data.
//!
//! Values can represent:
//!
//! - Null and boolean values.
//! - Signed and unsigned integers.
//! - Floating-point numbers.
//! - UTF-8 strings.
//! - Byte sequences.
//! - Ordered arrays.
//! - Objects with string keys.
//! - Collections of unique values.
//!
//! Objects use [`BTreeMap`] to maintain deterministic key
//! ordering, while sets use [`BTreeSet`] to maintain ordered,
//! unique elements.
//!
//! [`Value`] is independent of any serialization framework
//! and can be used as the common data representation for
//! parsing, formatting, configuration management, and
//! other CBR-related operations.
//!
//! # Examples
//!
//! ```
//! use ciber_cbr::value::Value;
//!
//! let mut settings = Value::empty_object();
//!
//! let inserted = settings.insert("enabled", Value::Bool(true));
//!
//! assert!(matches!(inserted, Ok(None)));
//! assert_eq!(
//!   settings.as_object().and_then(|object| object.get("enabled")),
//!   Some(&Value::Bool(true))
//! );
//! ```

use std::{
  cmp::Ordering,
  collections::{BTreeMap, BTreeSet},
};

/// An owned value in the CBR data format.
///
/// This enum provides a common representation for scalar
/// values and compound data structures.
///
/// It supports null values, booleans, signed and unsigned
/// integers, floating-point numbers, strings, byte sequences,
/// arrays, objects, and sets.
///
/// Objects use [`BTreeMap`] to maintain deterministic key
/// ordering. Sets use [`BTreeSet`] to maintain ordered,
/// unique elements.
///
/// # Equality and ordering
///
/// [`Value`] implements [`PartialEq`], [`Eq`], [`PartialOrd`],
/// and [`Ord`].
///
/// Variants are ordered according to their assigned ranks
/// in the implementation. Values of the same variant are
/// compared according to their underlying data.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
///
/// use ciber_cbr::value::Value;
///
/// let value = Value::Object(BTreeMap::from([
///   ("enabled".to_owned(), Value::Bool(true)),
///   ("retries".to_owned(), Value::I64(3)),
/// ]));
///
/// assert!(value.is_object());
/// assert_eq!(
///   value.as_object().and_then(|object| object.get("retries")),
///   Some(&Value::I64(3))
/// );
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
  /// Represents the absence of a value.
  Null,

  /// Represents a boolean value.
  Bool(bool),

  /// Represents a signed 64-bit integer.
  I64(i64),

  /// Represents an unsigned 64-bit integer.
  U64(u64),

  /// Represents a 64-bit floating-point number.
  ///
  /// Equality and ordering use [`f64::total_cmp`], providing
  /// consistent comparisons for all floating-point bit patterns,
  /// including NaN values and signed zero.
  F64(f64),

  /// Represents a signed 128-bit integer.
  I128(i128),

  /// Represents an unsigned 128-bit integer.
  U128(u128),

  /// Represents an owned UTF-8 string.
  String(String),

  /// Represents an opaque sequence of bytes.
  ///
  /// This variant stores raw bytes without imposing a
  /// particular textual encoding.
  Bytes(Vec<u8>),

  /// Represents an ordered sequence of values.
  ///
  /// Elements retain their insertion order and may contain
  /// duplicate values.
  Array(Vec<Self>),

  /// Represents an object containing string-keyed values.
  ///
  /// Entries are stored in a [`BTreeMap`], providing
  /// deterministic key ordering.
  Object(BTreeMap<String, Self>),

  /// Represents a collection of unique values.
  ///
  /// Elements are stored in a [`BTreeSet`], which maintains
  /// their order and automatically discards duplicates.
  Set(BTreeSet<Self>),
}

impl Value {
  /// Creates an empty object value.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let value = Value::empty_object();
  ///
  /// assert!(value.is_object());
  /// assert!(value.as_object().is_some_and(|object| object.is_empty()));
  /// ```
  #[must_use]
  pub const fn empty_object() -> Self { Self::Object(BTreeMap::new()) }

  /// Returns a reference to the value associated with `key`.
  ///
  /// Returns `None` if this value is not an object or the key
  /// does not exist.
  #[must_use]
  pub fn get(&self, key: &str) -> Option<&Self> { self.as_object()?.get(key) }

  /// Returns a mutable reference to the value associated with `key`.
  ///
  /// Returns `None` if this value is not an object or the key
  /// does not exist.
  #[must_use]
  pub fn get_mut(&mut self, key: &str) -> Option<&mut Self> {
    self.as_object_mut()?.get_mut(key)
  }

  /// Returns whether this object contains `key`.
  ///
  /// Returns `false` if this value is not an object.
  #[must_use]
  pub fn contains_key(&self, key: &str) -> bool {
    self.as_object().is_some_and(|object| object.contains_key(key))
  }

  /// Removes and returns the value associated with `key`.
  ///
  /// Returns `None` if this value is not an object or the key
  /// does not exist.
  #[must_use]
  pub fn remove(&mut self, key: &str) -> Option<Self> {
    self.as_object_mut()?.remove(key)
  }

  /// Returns the byte data if this value is a byte sequence.
  #[must_use]
  pub const fn as_bytes(&self) -> Option<&[u8]> {
    match self {
      Self::Bytes(bytes) => Some(bytes.as_slice()),
      _ => None,
    }
  }

  /// Returns mutable byte data if this value is a byte sequence.
  #[must_use]
  pub const fn as_bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
    match self {
      Self::Bytes(bytes) => Some(bytes),
      _ => None,
    }
  }

  /// Returns the set if this value is a set.
  #[must_use]
  pub const fn as_set(&self) -> Option<&BTreeSet<Self>> {
    match self {
      Self::Set(set) => Some(set),
      _ => None,
    }
  }

  /// Returns the mutable set if this value is a set.
  #[must_use]
  pub const fn as_set_mut(&mut self) -> Option<&mut BTreeSet<Self>> {
    match self {
      Self::Set(set) => Some(set),
      _ => None,
    }
  }

  /// Returns whether this value is an object.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// assert!(Value::empty_object().is_object());
  /// assert!(!Value::Null.is_object());
  /// ```
  #[must_use]
  pub const fn is_object(&self) -> bool { matches!(self, Self::Object(_)) }

  /// Returns whether this value is [`Value::Null`].
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// assert!(Value::Null.is_null());
  /// assert!(!Value::Bool(false).is_null());
  /// ```
  #[must_use]
  pub const fn is_null(&self) -> bool { matches!(self, Self::Null) }

  /// Returns a reference to the object's entries.
  ///
  /// Returns `None` if this value is not an object.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let value = Value::empty_object();
  ///
  /// assert!(value.as_object().is_some());
  /// assert!(Value::Null.as_object().is_none());
  /// ```
  #[must_use]
  pub const fn as_object(&self) -> Option<&BTreeMap<String, Self>> {
    match self {
      Self::Object(object) => Some(object),
      _ => None,
    }
  }

  /// Returns a mutable reference to the object's entries.
  ///
  /// Returns `None` if this value is not an object.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let mut value = Value::empty_object();
  ///
  /// if let Some(object) = value.as_object_mut() {
  ///   object.insert("enabled".to_owned(), Value::Bool(true));
  /// }
  ///
  /// assert!(
  ///   value.as_object().is_some_and(|object| object.contains_key("enabled"))
  /// );
  /// ```
  #[must_use]
  pub const fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String, Self>> {
    match self {
      Self::Object(object) => Some(object),
      _ => None,
    }
  }

  /// Returns a slice containing the array's elements.
  ///
  /// Returns `None` if this value is not an array.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let value = Value::Array(vec![Value::Bool(true)]);
  ///
  /// assert_eq!(value.as_array().map(<[_]>::len), Some(1));
  /// assert!(Value::Null.as_array().is_none());
  /// ```
  #[must_use]
  pub fn as_array(&self) -> Option<&[Self]> {
    match self {
      Self::Array(values) => Some(values),
      _ => None,
    }
  }

  /// Returns a mutable reference to the array's elements.
  ///
  /// Returns `None` if this value is not an array.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let mut value = Value::Array(Vec::new());
  ///
  /// if let Some(values) = value.as_array_mut() {
  ///   values.push(Value::Bool(true));
  /// }
  ///
  /// assert_eq!(value.as_array().map(<[_]>::len), Some(1));
  /// ```
  #[must_use]
  pub const fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
    match self {
      Self::Array(values) => Some(values),
      _ => None,
    }
  }

  /// Returns the string slice if this value is a string.
  ///
  /// Returns `None` for every other variant.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let value = Value::from("Cirious");
  ///
  /// assert_eq!(value.as_str(), Some("Cirious"));
  /// assert_eq!(Value::Null.as_str(), None);
  /// ```
  #[must_use]
  pub fn as_str(&self) -> Option<&str> {
    match self {
      Self::String(value) => Some(value),
      _ => None,
    }
  }

  /// Returns this value as an `i64`, if representable.
  ///
  /// Signed and unsigned integer variants are converted when
  /// their values fit within the range of `i64`.
  ///
  /// Floating-point values and nonnumeric variants return `None`.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// assert_eq!(Value::I64(-7).as_i64(), Some(-7));
  /// assert_eq!(Value::U64(7).as_i64(), Some(7));
  /// assert_eq!(Value::I128(i128::MAX).as_i64(), None);
  /// assert_eq!(Value::U128(u128::MAX).as_i64(), None);
  /// assert_eq!(Value::String("7".to_owned()).as_i64(), None);
  /// ```
  #[must_use]
  pub fn as_i64(&self) -> Option<i64> {
    match self {
      Self::I64(value) => Some(*value),
      Self::I128(value) => i64::try_from(*value).ok(),
      Self::U64(value) => i64::try_from(*value).ok(),
      Self::U128(value) => i64::try_from(*value).ok(),
      _ => None,
    }
  }

  /// Returns the value as an unsigned 64-bit integer.
  ///
  /// Integer variants are converted when the value is within the
  /// valid range of `u64`. Floating-point values are not converted.
  ///
  /// # Returns
  ///
  /// `Some` if the value can be represented as `u64`, otherwise `None`.
  #[must_use]
  pub fn as_u64(&self) -> Option<u64> {
    match self {
      Self::I64(value) => u64::try_from(*value).ok(),
      Self::U64(value) => Some(*value),
      Self::I128(value) => u64::try_from(*value).ok(),
      Self::U128(value) => u64::try_from(*value).ok(),
      _ => None,
    }
  }

  /// Returns the value as a signed 128-bit integer.
  ///
  /// Integer variants are converted when the value is within the
  /// valid range of `i128`. Floating-point values are not converted.
  ///
  /// # Returns
  ///
  /// `Some` if the value can be represented as `i128`, otherwise `None`.
  #[must_use]
  pub fn as_i128(&self) -> Option<i128> {
    match self {
      Self::I64(value) => Some(i128::from(*value)),
      Self::U64(value) => Some(i128::from(*value)),
      Self::I128(value) => Some(*value),
      Self::U128(value) => i128::try_from(*value).ok(),
      _ => None,
    }
  }

  /// Returns the value as an unsigned 128-bit integer.
  ///
  /// Integer variants are converted when the value is within the
  /// valid range of `u128`. Floating-point values are not converted.
  ///
  /// # Returns
  ///
  /// `Some` if the value can be represented as `u128`, otherwise `None`.
  #[must_use]
  pub fn as_u128(&self) -> Option<u128> {
    match self {
      Self::I64(value) => u128::try_from(*value).ok(),
      Self::U64(value) => Some(u128::from(*value)),
      Self::I128(value) => u128::try_from(*value).ok(),
      Self::U128(value) => Some(*value),
      _ => None,
    }
  }

  /// Returns the value as a 64-bit floating-point number.
  ///
  /// Integer variants are converted to `f64`. Large integers may lose
  /// precision because `f64` cannot represent every integer exactly.
  ///
  /// # Returns
  ///
  /// `Some` if the value is numeric, otherwise `None`.
  #[must_use]
  #[allow(clippy::cast_precision_loss)]
  pub const fn as_f64(&self) -> Option<f64> {
    match self {
      Self::I64(value) => Some(*value as f64),
      Self::U64(value) => Some(*value as f64),
      Self::I128(value) => Some(*value as f64),
      Self::U128(value) => Some(*value as f64),
      Self::F64(value) => Some(*value),
      _ => None,
    }
  }

  /// Returns the boolean value if this value is a boolean.
  ///
  /// Returns `None` for every other variant.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// assert_eq!(Value::Bool(true).as_bool(), Some(true));
  /// assert_eq!(Value::Null.as_bool(), None);
  /// ```
  #[must_use]
  pub const fn as_bool(&self) -> Option<bool> {
    match self {
      Self::Bool(value) => Some(*value),
      _ => None,
    }
  }

  /// Inserts a key-value pair into this object.
  ///
  /// If the key already exists, its previous value is returned
  /// and replaced by the supplied value.
  ///
  /// Returns `Err(value)` if `self` is not an object.
  /// In that case, `self` remains unchanged.
  ///
  /// # Arguments
  ///
  /// - `key`: The object property name.
  /// - `value`: The value to insert.
  ///
  /// # Errors
  ///
  /// Returns the supplied value if `self` is not an object.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let mut object = Value::empty_object();
  ///
  /// let inserted = object.insert("name", Value::from("CiberCodes"));
  /// assert!(matches!(inserted, Ok(None)));
  ///
  /// let replaced = object.insert("name", Value::from("Cirious"));
  ///
  /// assert!(matches!(
  ///   replaced,
  ///   Ok(Some(Value::String(previous))) if previous == "CiberCodes"
  /// ));
  /// ```
  ///
  /// Inserting into a non-object returns the supplied value:
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let mut value = Value::Null;
  ///
  /// let result = value.insert("name", Value::Bool(true));
  ///
  /// assert!(matches!(result, Err(Value::Bool(true))));
  /// assert!(value.is_null());
  /// ```
  pub fn insert(
    &mut self,
    key: impl Into<String>,
    value: Self,
  ) -> Result<Option<Self>, Self> {
    match self {
      Self::Object(object) => Ok(object.insert(key.into(), value)),
      _ => Err(value),
    }
  }
}

/// Compares two values for equality.
///
/// Equality is determined by the total ordering implemented
/// by [`Ord`]. Two values are equal when their comparison
/// returns [`Ordering::Equal`].
///
/// This ensures that equality is consistent with the ordering
/// of all [`Value`] variants, including floating-point values.
impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool { self.cmp(other).is_eq() }
}

/// Marks [`Value`] as a type with total equality.
///
/// Every value can be compared for equality with every other
/// value, and the equality relation is consistent with the
/// total ordering implemented by [`Ord`].
impl Eq for Value {}

/// Provides partial comparison between two values.
///
/// Since [`Value`] implements [`Ord`], every pair of values
/// has a defined ordering. This implementation therefore
/// always returns `Some` containing the result of the total
/// comparison.
impl PartialOrd for Value {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

/// Provides a total ordering for CBR values.
///
/// Values of different variants are compared according to
/// their assigned ranks in the implementation. Values of the
/// same variant are compared according to their underlying
/// data.
///
/// Floating-point values are compared using
/// [`f64::total_cmp`], which provides a total ordering for
/// all floating-point bit patterns, including NaN values
/// and signed zero.
///
/// Compound values follow the ordering of their underlying
/// collections:
///
/// - Arrays are compared lexicographically, preserving element order.
/// - Objects are compared according to their ordered key-value entries.
/// - Sets are compared according to their ordered elements.
///
/// This ordering is consistent with the [`PartialEq`]
/// and [`Eq`] implementations.
///
/// # Examples
///
/// ```
/// use std::cmp::Ordering;
///
/// use ciber_cbr::value::Value;
///
/// let first = Value::I64(10);
/// let second = Value::I64(20);
///
/// assert_eq!(first.cmp(&second), Ordering::Less);
///
/// let null = Value::Null;
/// let boolean = Value::Bool(false);
///
/// assert_eq!(null.cmp(&boolean), Ordering::Less);
/// assert_eq!(first, Value::I64(10));
/// ```
impl Ord for Value {
  fn cmp(&self, other: &Self) -> Ordering {
    let rank = |value: &Self| match value {
      Self::Null => 0,
      Self::Bool(_) => 1,
      Self::I64(_) => 2,
      Self::U64(_) => 3,
      Self::F64(_) => 4,
      Self::I128(_) => 5,
      Self::U128(_) => 6,
      Self::String(_) => 7,
      Self::Bytes(_) => 8,
      Self::Array(_) => 9,
      Self::Object(_) => 10,
      Self::Set(_) => 11,
    };

    match rank(self).cmp(&rank(other)) {
      Ordering::Equal => {}
      ordering => return ordering,
    }

    match (self, other) {
      (Self::Null, Self::Null) => Ordering::Equal,
      (Self::Bool(a), Self::Bool(b)) => a.cmp(b),
      (Self::I64(a), Self::I64(b)) => a.cmp(b),
      (Self::U64(a), Self::U64(b)) => a.cmp(b),
      (Self::F64(a), Self::F64(b)) => a.total_cmp(b),
      (Self::I128(a), Self::I128(b)) => a.cmp(b),
      (Self::U128(a), Self::U128(b)) => a.cmp(b),
      (Self::String(a), Self::String(b)) => a.cmp(b),
      (Self::Bytes(a), Self::Bytes(b)) => a.cmp(b),
      (Self::Array(a), Self::Array(b)) => a.cmp(b),
      (Self::Object(a), Self::Object(b)) => a.cmp(b),
      (Self::Set(a), Self::Set(b)) => a.cmp(b),
      _ => unreachable!("Value variants have different ranks"),
    }
  }
}

/// Converts a boolean into a CBR value.
impl From<bool> for Value {
  /// Converts the boolean into [`Value::Bool`].
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let value = Value::from(true);
  ///
  /// assert_eq!(value.as_bool(), Some(true));
  /// ```
  fn from(value: bool) -> Self { Self::Bool(value) }
}

/// Converts a signed 64-bit integer into a CBR value.
impl From<i64> for Value {
  /// Converts the integer into [`Value::I64`].
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// assert!(matches!(Value::from(-5_i64), Value::I64(-5)));
  /// ```
  fn from(value: i64) -> Self { Self::I64(value) }
}

/// Converts an unsigned 64-bit integer into a CBR value.
impl From<u64> for Value {
  /// Converts the integer into [`Value::U64`].
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// assert!(matches!(Value::from(5_u64), Value::U64(5)));
  /// ```
  fn from(value: u64) -> Self { Self::U64(value) }
}

/// Converts a signed 128-bit integer into a CBR value.
impl From<i128> for Value {
  /// Converts the integer into [`Value::I128`].
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// assert!(matches!(
  ///     Value::from(i128::MAX),
  ///     Value::I128(value) if value == i128::MAX
  /// ));
  /// ```
  fn from(value: i128) -> Self { Self::I128(value) }
}

/// Converts an unsigned 128-bit integer into a CBR value.
impl From<u128> for Value {
  /// Converts the integer into [`Value::U128`].
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// assert!(matches!(
  ///     Value::from(u128::MAX),
  ///     Value::U128(value) if value == u128::MAX
  /// ));
  /// ```
  fn from(value: u128) -> Self { Self::U128(value) }
}

/// Converts a 64-bit floating-point number into a CBR value.
impl From<f64> for Value {
  /// Converts the number into [`Value::F64`].
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// assert!(matches!(
  ///     Value::from(2.5_f64),
  ///     Value::F64(value) if value == 2.5
  /// ));
  /// ```
  fn from(value: f64) -> Self { Self::F64(value) }
}

/// Converts an owned string into a CBR value.
impl From<String> for Value {
  /// Converts the string into [`Value::String`].
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let value = Value::from(String::from("Cirious"));
  ///
  /// assert_eq!(value.as_str(), Some("Cirious"));
  /// ```
  fn from(value: String) -> Self { Self::String(value) }
}

/// Converts a string slice into an owned CBR string value.
impl From<&str> for Value {
  /// Copies the string slice into [`Value::String`].
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let value = Value::from("Cirious");
  ///
  /// assert_eq!(value.as_str(), Some("Cirious"));
  /// ```
  fn from(value: &str) -> Self { Self::String(value.to_owned()) }
}

/// Converts a vector of values into a CBR array.
impl<T> From<Vec<T>> for Value
where
  T: Into<Self>,
{
  /// Converts each element into a CBR value and collects
  /// the results into [`Value::Array`].
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let value = Value::from(vec![1_i64, 2_i64, 3_i64]);
  ///
  /// assert_eq!(value.as_array().map(<[_]>::len), Some(3));
  /// ```
  fn from(values: Vec<T>) -> Self {
    Self::Array(values.into_iter().map(Into::into).collect())
  }
}

/// Provides an empty object as the default CBR value.
impl Default for Value {
  /// Creates an empty object.
  ///
  /// # Examples
  ///
  /// ```
  /// use ciber_cbr::value::Value;
  ///
  /// let value = Value::default();
  ///
  /// assert!(value.is_object());
  /// assert!(value.as_object().is_some_and(|object| object.is_empty()));
  /// ```
  fn default() -> Self { Self::empty_object() }
}
