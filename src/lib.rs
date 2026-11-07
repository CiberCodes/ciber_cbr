#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "crypto")]
#[cfg_attr(docsrs, doc(cfg(feature = "crypto")))]
pub mod crypto;
pub mod error;
pub mod file;
pub mod formatter;
pub mod lexer;
pub mod parser;
pub mod value;

pub use error::{Error, ErrorKind, Position, Result};
pub use file::{CbrFile, Options};
pub use parser::ParseLimits;
pub use value::Value;
