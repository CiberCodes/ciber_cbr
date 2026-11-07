//! Demonstrates compact and pretty CBR formatting.

use std::error::Error;

use ciber_cbr::{CbrFile, Value, formatter};

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = Value::empty_object();

    let _ = database.insert("host", Value::from("localhost"));
    let _ = database.insert("port", Value::from(5432_u64));

    let mut application = Value::empty_object();
    let _ = application.insert("name", Value::from("CiberApp"));
    let _ = application.insert("database", database);

    let file = CbrFile::from_value(application);

    let compact = file.to_string()?;
    let pretty = file.to_string_pretty()?;

    println!("Compact representation:\n{compact}");
    println!();
    println!("Pretty representation:\n{pretty}");

    // The formatter can also operate directly on Value.
    let formatted = formatter::to_string(file.value())?;
    println!("\nFormatted directly from Value:\n{formatted}");

    Ok(())
}
