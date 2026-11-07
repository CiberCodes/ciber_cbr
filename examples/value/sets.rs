//! Demonstrates CBR set values.

use std::collections::BTreeSet;

use ciber_cbr::Value;

fn main() {
    let mut tags = BTreeSet::new();

    tags.insert(Value::from("rust"));
    tags.insert(Value::from("cbr"));
    tags.insert(Value::from("crypto"));
    tags.insert(Value::from("rust"));

    let mut value = Value::Set(tags);

    println!("Set: {value:?}");

    if let Some(set) = value.as_set() {
        println!("Unique elements: {}", set.len());

        for item in set {
            println!("- {item:?}");
        }
    }

    if let Some(set) = value.as_set_mut() {
        set.insert(Value::from("ciber-codes"));
    }

    println!("Updated set: {value:?}");
}
