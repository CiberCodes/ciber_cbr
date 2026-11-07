//! Demonstrates creating and writing a CBR document.

use std::error::Error;

use ciber_cbr::{CbrFile, Value};

fn main() -> Result<(), Box<dyn Error>> {
  let mut application = Value::empty_object();

  let _ = application.insert("name", Value::from("CiberApp"));
  let _ = application.insert("version", Value::from(1_u64));
  let _ = application.insert("enabled", Value::from(true));

  let file = CbrFile::from_value(application);

  file.save("examples/Resources/application.cbr")?;

  println!("CBR document written successfully.");

  Ok(())
}
