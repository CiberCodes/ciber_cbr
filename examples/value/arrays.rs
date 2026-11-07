//! Demonstrates CBR array values.

use ciber_cbr::Value;

fn main() {
    let mut numbers = Value::Array(vec![
        Value::from(10_i64),
        Value::from(20_i64),
        Value::from(30_i64),
    ]);

    println!("Array: {numbers:?}");

    if let Some(values) = numbers.as_array() {
        println!("Array length: {}", values.len());
        println!("First element: {:?}", values.first());
    }

    if let Some(values) = numbers.as_array_mut() {
        values.push(Value::from(40_i64));
    }

    println!("Modified array: {numbers:?}");
}
