//! Demonstrates reading a CBR document from a file.

use std::error::Error;

use ciber_cbr::CbrFile;

fn main() -> Result<(), Box<dyn Error>> {
    let file = CbrFile::open("examples/Resources/application.cbr")?;

    println!("Document contents:");
    println!("{}", file.to_string_pretty()?);

    Ok(())
}
