//! Boa's **`boa_unicode`** crate for query valid Unicode identifiers.
//!
//! # Crate Overview
//! This crate implements the extension to query if a char belongs to a particular unicode identifier property.
//!
//! Current Version:
//!  - Unicode 15.0.0
//!
//! More information:
//!  - [Unicode® Standard Annex #31][uax31]
//!
//! # About Boa
//! Boa is an open-source, experimental ECMAScript Engine written in Rust for lexing, parsing and executing ECMAScript/JavaScript. Currently, Boa
//! supports some of the [language][boa-conformance]. More information can be viewed at [Boa's website][boa-web].
//!
//! Try out the most recent release with Boa's live demo [playground][boa-playground].  
//!
//! # Boa Crates
//!  - **`boa_ast`** - Boa's ECMAScript Abstract Syntax Tree.
//!  - **`boa_engine`** - Boa's implementation of ECMAScript builtin objects and execution.
//!  - **`boa_gc`** - Boa's garbage collector.
//!  - **`boa_interner`** - Boa's string interner.
//!  - **`boa_parser`** - Boa's lexer and parser.
//!  - **`boa_profiler`** - Boa's code profiler.
//!  - **`boa_unicode`** - Boa's Unicode identifier.
//!  - **`boa_icu_provider`** - Boa's ICU4X data provider.
//!
//! [uax31]: http://unicode.org/reports/tr31
//! [boa-conformance]: https://boajs.dev/boa/test262/
//! [boa-web]: https://boajs.dev/
//! [boa-playground]: https://boajs.dev/boa/playground/

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
#![allow(clippy::redundant_pub_crate)]
#![no_std]

mod tables;
#[cfg(test)]
mod tests;

use unicode_general_category::{get_general_category, GeneralCategory};

/// The version of Unicode.
pub const UNICODE_VERSION: (u64, u64, u64) = (15, 0, 0);

/// Extend a type of code point to query if a value belongs to a particular Unicode property.
///
/// This trait defines methods for querying properties and classes mentioned or defined in Unicode® Standard Annex #31.
/// These properties are used to determine if a code point (char) is valid for being the start/part of an identifier and assist in
/// the standard treatment of Unicode identifiers in parsers and lexers.
///
/// More information:
///  - [Unicode® Standard Annex #31][uax31]
///
/// [uax31]: http://unicode.org/reports/tr31
pub trait UnicodeProperties: Sized + Copy {
    /// Returns `true` if this value is a member of `ID_Start`.
    fn is_id_start(self) -> bool;

    /// Returns `true` if this value is a member of `ID_Continue`.
    fn is_id_continue(self) -> bool;

    /// Returns `true` if this value is a member of `Other_ID_Start`.
    fn is_other_id_start(self) -> bool;

    /// Returns `true` if this value is a member of `Other_ID_Continue`.
    fn is_other_id_continue(self) -> bool;

    /// Returns `true` if this value is a member of `Pattern_Syntax`.
    fn is_pattern_syntax(self) -> bool;

    /// Returns `true` if this value is a member of `Pattern_White_Space`.
    fn is_pattern_whitespace(self) -> bool;
}

fn table_binary_search(target: char, table: &'static [char]) -> bool {
    table.binary_search(&target).is_ok()
}

impl UnicodeProperties for char {
    #[inline]
    fn is_id_start(self) -> bool {
        !self.is_pattern_syntax()
            && !self.is_pattern_whitespace()
            && (self.is_other_id_start()
                || matches!(
                    get_general_category(self),
                    GeneralCategory::LowercaseLetter
                        | GeneralCategory::ModifierLetter
                        | GeneralCategory::OtherLetter
                        | GeneralCategory::TitlecaseLetter
                        | GeneralCategory::UppercaseLetter
                        | GeneralCategory::LetterNumber
                ))
    }

    #[inline]
    fn is_id_continue(self) -> bool {
        !self.is_pattern_syntax()
            && !self.is_pattern_whitespace()
            && (self.is_id_start()
                || self.is_other_id_continue()
                || matches!(
                    get_general_category(self),
                    GeneralCategory::NonspacingMark
                        | GeneralCategory::SpacingMark
                        | GeneralCategory::DecimalNumber
                        | GeneralCategory::ConnectorPunctuation
                ))
    }

    #[inline]
    fn is_other_id_start(self) -> bool {
        table_binary_search(self, tables::OTHER_ID_START)
    }
    #[inline]
    fn is_other_id_continue(self) -> bool {
        table_binary_search(self, tables::OTHER_ID_CONTINUE)
    }
    #[inline]
    fn is_pattern_syntax(self) -> bool {
        table_binary_search(self, tables::PATTERN_SYNTAX)
    }
    #[inline]
    fn is_pattern_whitespace(self) -> bool {
        table_binary_search(self, tables::PATTERN_WHITE_SPACE)
    }
}
