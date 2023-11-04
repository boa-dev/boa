//! Boa's `boa_temporal` crate is intended to serve as an engine agnostic
//! implementation the ECMAScript's Temporal builtin and algorithm.
#![doc = include_str!("../../ABOUT.md")]
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
    // Currently throws a false positive regarding dependencies that are only used in benchmarks.
    unused_crate_dependencies,
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::missing_errors_doc,
    clippy::let_unit_value,
    clippy::option_if_let_else,

    // It may be worth to look if we can fix the issues highlighted by these lints.
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,

    // Add temporarily - Needs addressing
    clippy::missing_panics_doc,
)]

pub mod calendar;
pub mod date;
pub mod datetime;
pub mod duration;
pub mod error;
pub mod fields;
pub mod iso;
pub mod month_day;
pub mod options;
pub mod time;
pub(crate) mod utils;
pub mod year_month;
pub mod zoneddatetime;

use num_bigint::BigInt;
// TODO: evaluate positives and negatives of using tinystr.
// Re-exporting tinystr as a convenience, as it is currently tied into the API.
pub use tinystr::{TinyAsciiStr, TinyStr16, TinyStr4, TinyStr8};

pub use error::TemporalError;

/// The `Temporal` result type
pub type TemporalResult<T> = Result<T, TemporalError>;

// Relavant numeric constants
/// Nanoseconds per day constant: 8.64e+13
pub(crate) const NS_PER_DAY: i64 = 86_400_000_000_000;
/// Milliseconds per day constant: 8.64e+7
pub(crate) const MS_PER_DAY: i32 = 24 * 60 * 60 * 1000;

pub(crate) fn ns_max_instant() -> BigInt {
    BigInt::from(i128::from(NS_PER_DAY) * 100_000_000i128)
}

pub(crate) fn ns_min_instant() -> BigInt {
    ns_max_instant() * -1
}
