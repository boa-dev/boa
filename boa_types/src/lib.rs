//! Boa's **`boa_interner`** is a string interner for compiler performance.
//!
//! # Crate Overview
//!
//! The idea behind using a string interner is that in most of the code, strings such as
//! identifiers and literals are often repeated. This causes extra burden when comparing them and
//! storing them. A string interner stores a unique `usize` symbol for each string, making sure
//! that there are no duplicates. This makes it much easier to compare, since it's just comparing
//! to `usize`, and also it's easier to store, since instead of a heap-allocated string, you only
//! need to store a `usize`. This reduces memory consumption and improves performance in the
//! compiler.
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(
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
    missing_docs,
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

    // clippy allowed by default
    clippy::dbg_macro,

    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
)]
#![allow(
    clippy::redundant_pub_crate,
    // TODO deny once false positive is fixed (https://github.com/rust-lang/rust-clippy/issues/9626).
    clippy::trait_duplication_in_bounds
)]

/// Helper function to check if a `char` is trimmable.
#[must_use]
pub const fn is_trimmable_whitespace(c: char) -> bool {
    // The rust implementation of `trim` does not regard the same characters whitespace as ecma standard does
    //
    // Rust uses \p{White_Space} by default, which also includes:
    // `\u{0085}' (next line)
    // And does not include:
    // '\u{FEFF}' (zero width non-breaking space)
    // Explicit whitespace: https://tc39.es/ecma262/#sec-white-space
    matches!(
        c,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{FEFF}' |
    // Unicode Space_Separator category
    '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' |
    // Line terminators: https://tc39.es/ecma262/#sec-line-terminators
    '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}'
    )
}

pub mod string;
pub mod symbol;
pub(crate) mod tagged;
