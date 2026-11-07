//! Demonstrates CBR object values.

use std::collections::BTreeMap;

use ciber_cbr::Value;

fn main() {
    let mut database = BTreeMap::new();

    database.insert("host".to_owned(), Value::from("localhost"));
    database.insert("port".to_owned(), Value::from(5432_u64));

    let mut application = Value::empty_object();

    let _ = application.insert("name", Value::from("CiberApp"));
    let _ = application.insert("version", Value::from(1_u64));
    let _ = application.insert("database", Value::Object(database));

    println!("Application: {application:#?}");

    if let Some(name) = application.get("name").and_then(Value::as_str) {
        println!("Application name: {name}");
    }

    if let Some(database) = application.get("database")
        && let Some(host) = database.get("host").and_then(Value::as_str)
    {
        println!("Database host: {host}");
    }

    let _ = application.insert("enabled", Value::from(true));
    let _ = application.remove("version");

    println!("Updated application: {application:#?}");
}
