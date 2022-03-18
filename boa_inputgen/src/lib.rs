//! Input generator for Boa.
//!
//! Generating JavaScript code is tough, but generating ASTs is easy! Since Boa offers a mechanism
//! for converting from AST to source code, we can use [arbitrary](https://docs.rs/arbitrary) to
//! generate the valid AST and convert it back into source code.
//!
//! There is a snag, however; the interner expects identifiers to replace the
//! [Sym](boa_interner::Sym) instances, but we've generated Syms with arbitrary IDs. To get around
//! this, we walk the AST to replace all identifiers in the AST with valid identifiers as generated
//! by arbitrary.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(
    clippy::perf,
    clippy::single_match_else,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::struct_excessive_bools,
    clippy::doc_markdown,
    clippy::semicolon_if_nothing_returned,
    clippy::pedantic
)]
#![deny(
    clippy::all,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::use_self,
    clippy::unnested_or_patterns,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_unwrap_or,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    // rustdoc,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_ptr_alignment,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions,
    clippy::let_unit_value,
    rustdoc::missing_doc_code_examples
)]

pub use data::FuzzData;
pub(crate) use ident_walk::replace_syms;

mod data;
mod ident_walk;
