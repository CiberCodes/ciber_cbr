<div style="text-align: center;">

# ⚙️ Ciber CBR

**Structured data representation and document handling for the Cirious ecosystem**

[![CI](https://github.com/CiberCodes/ciber_cbr/actions/workflows/ci.yml/badge.svg)](https://github.com/CiberCodes/ciber_cbr/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ciber_cbr.svg)](https://crates.io/crates/ciber_cbr)
[![Documentation](https://docs.rs/ciber_cbr/badge.svg)](https://docs.rs/ciber_cbr)
[![Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](#-license)

</div>

---

## 📖 Overview

**Ciber CBR** is a data format library developed for the **`CiberCodes`** and **Cirious** ecosystems.

It provides a structured representation of CBR (Ciber Configuration Representation) values, together with lexical analysis, parsing, formatting, and document handling.

The library is designed as an independent core, without requiring a serialization framework. This allows other projects to use the CBR data model and its document-processing capabilities independently.

## ✨ Features

- **Structured values:** represent null, booleans, signed and unsigned integers, floating-point numbers, strings, bytes, arrays, objects, and sets through [`Value`](src/value.rs).
- **Lexical analysis:** tokenize CBR input, including identifiers, strings, numbers, delimiters, and comments.
- **Document parsing:** parse CBR documents into structured values.
- **Formatting:** generate compact and pretty-printed CBR representations.
- **Document handling:** create, read, write, and save CBR documents through [`CbrFile`](src/file.rs).
- **Deterministic collections:** use ordered maps and sets for predictable output.
- **Optional Serde derives:** enable Serde support for compatible core data types.
- **Consistent errors:** use the library's error, error-kind, position, and result types.

## 📦 Installation

Add Ciber CBR to your `Cargo.toml`:

```toml
[dependencies]
ciber_cbr = "0.1"
```

### Optional Serde support

Serde derives are disabled by default. Enable the `serde` feature if you need them:

```toml
[dependencies]
ciber_cbr = { version = "0.1", features = ["serde"] }
```

The core library does not implement the CBR Serde serializer or deserializer. Those integrations are intended for a separate `serde_cbr` crate.

## 🚀 Quick Start

Parsing APIs use `ParseLimits::default()` by default: 16 MiB maximum input,
128 nested containers, 100,000 elements per array or set, and 1 MiB per decoded
string (UTF-8 bytes). Use `CbrFile::parse_with_limits` or
`CbrFile::from_reader_with_limits` to choose other limits. A zero limit forbids
the corresponding resource.

### Parsing a CBR document

```rust
use ciber_cbr::{
    CbrFile,
    lexer::tokenize,
    parser::Parser,
};

fn main() -> ciber_cbr::Result<()> {
    let source = r#"
        name = "Cirious"
        version = "0.1.0"
        enabled = true
    "#;

    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    let value = parser.parse()?;

    let document = CbrFile::from_value(value);

    println!("{}", document.to_string_pretty()?);

    Ok(())
}
```

### Creating structured values

```rust
use ciber_cbr::Value;

fn main() {
    let host = Value::String("localhost".to_owned());
    let port = Value::I64(5432);

    println!("Host: {host:?}");
    println!("Port: {port:?}");
}
```

### Reading and writing documents

```rust
use ciber_cbr::CbrFile;

fn main() -> ciber_cbr::Result<()> {
    std::fs::write("config.cbr", r#"name = "Cirious""#)?;
    let document = CbrFile::open("config.cbr")?;

    document.save_pretty("output.cbr")?;
    std::fs::remove_file("config.cbr")?;
    std::fs::remove_file("output.cbr")?;

    Ok(())
}
```

## 🧩 Feature Flags

| Feature | Description |
| ------- | ----------- |
| `serde` | Enables optional Serde derives for supported data types. |
| `crypto` | Enables SHA-256 helpers, HKDF-SHA256 key derivation, random key generation, and AES-256-GCM encryption. |
| `full` | Enables both `serde` and `crypto`. |

The default feature set is empty. No optional integration is enabled unless explicitly requested. AES-GCM ciphertext includes a 12-byte nonce prefix; callers must keep encryption keys secret and supply the same additional authenticated data when decrypting. Do not reuse a key/nonce pair.

## 🧪 Examples and Tests

The repository includes examples and integration tests covering the core functionality.

| Resource | Description |
| -------- | ----------- |
| [Basic example](examples/basic.rs) | Demonstrates basic CBR usage. |
| [Value tests](tests/value.rs) | Verify structured values, conversions, collections, and ordering. |
| [Lexer tests](tests/lexer.rs) | Verify tokenization, comments, strings, and source positions. |
| [Parser tests](tests/parse.rs) | Verify document parsing, values, and syntax errors. |
| [Formatter tests](tests/formatter.rs) | Verify compact formatting, pretty formatting, and round-trips. |
| [File tests](tests/file.rs) | Verify document reading, writing, and persistence. |

## 📚 Documentation

- [CBR format specification](docs/specification/grammar.md)
- [Benchmark instructions and methodology](benches/README.md)
- [API documentation on docs.rs](https://docs.rs/ciber_cbr)
- [Crates.io package](https://crates.io/crates/ciber_cbr)
- [GitHub repository](https://github.com/CiberCodes/ciber_cbr)
- [Feature flags](#-feature-flags)
- [Examples and tests](#-examples-and-tests)

## 📜 License

Licensed under either of the following, at your option:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

---

<div style="text-align: center;">

*Minimalist by design. Consistent in execution.*

<sub>Engineered by [CiberCodes](https://github.com/CiberCodes)</sub>

</div>
