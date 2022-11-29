//! Boa's **boa_parser** crate is a parser targeting the latest [ECMAScript language specification][spec].
//!
//! # Crate Overview
//! This crate contains implementations of a [`Lexer`] and a [`Parser`] for the **ECMAScript**
//! language. The [lexical grammar][lex] and the [syntactic grammar][grammar] being targeted are
//! fully defined in the specification. See the links provided for more information.
//!
//! # About Boa
//! Boa is an open-source, experimental ECMAScript Engine written in Rust for lexing, parsing and executing ECMAScript/JavaScript. Currently, Boa
//! supports some of the [language][boa-conformance]. More information can be viewed at [Boa's website][boa-web].
//!
//! Try out the most recent release with Boa's live demo [playground][boa-playground].  
//!
//! # Boa Crates
//!  - **boa_ast** - Boa's ECMAScript Abstract Syntax Tree.
//!  - **boa_engine** - Boa's implementation of ECMAScript builtin objects and execution.
//!  - **boa_gc** - Boa's garbage collector
//!  - **boa_interner** - Boa's string interner
//!  - **boa_parser** - Boa's lexer and parser
//!  - **boa_profiler** - Boa's code profiler
//!  - **boa_unicode** - Boa's Unicode identifier
//!
//! [spec]: https://tc39.es/ecma262
//! [lex]: https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar
//! [grammar]: https://tc39.es/ecma262/#sec-ecmascript-language-expressions
//! [boa-conformance]: https://boa-dev.github.io/boa/test262/
//! [boa-web]: https://boa-dev.github.io/
//! [boa-playground]: https://boa-dev.github.io/boa/playground/

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,

    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,

    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,

    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::let_unit_value,
    clippy::redundant_pub_crate
)]

pub mod error;
pub mod lexer;
pub mod parser;

pub use error::Error;
pub use lexer::Lexer;
pub use parser::Parser;
