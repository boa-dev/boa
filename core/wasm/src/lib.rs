//! Boa's **`boa_wasm_parser`** crate is a parser targeting the
//! [`WebAssembly` binary format specification][spec].
//!
//! # Crate Overview
//!
//! This crate contains an implementation of a [`Parser`] for `WebAssembly` binary
//! modules. The [binary format][binformat] being targeted is fully defined in the
//! specification. See the links provided for more information.
//!
//! [spec]: https://webassembly.github.io/spec/core/
//! [binformat]: https://webassembly.github.io/spec/core/binary/index.html
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![allow(clippy::module_name_repetitions, clippy::redundant_pub_crate)]

pub mod error;
pub mod parser;
pub mod types;

pub use error::Error;
pub use parser::Parser;
