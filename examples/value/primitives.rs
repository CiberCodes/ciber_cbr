//! Demonstrates primitive CBR values.

use ciber_cbr::Value;

fn main() {
  let null = Value::Null;
  let boolean = Value::from(true);
  let signed = Value::from(-42_i64);
  let unsigned = Value::from(42_u64);
  let floating = Value::from(std::f64::consts::PI);
  let text = Value::from("CiberCodes");

  println!("Null: {null:?}");
  println!("Boolean: {boolean:?}");
  println!("Signed integer: {signed:?}");
  println!("Unsigned integer: {unsigned:?}");
  println!("Floating-point: {floating:?}");
  println!("String: {text:?}");
}
