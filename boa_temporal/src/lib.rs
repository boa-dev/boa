//! Boa's `boa_temporal` crate is intended to serve as an engine agnostic
//! implementation the ECMAScript's Temporal builtin and algorithm.
#![doc = include_str!("../../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
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
pub mod instant;
pub mod iso;
pub mod month_day;
pub mod options;
pub mod time;
pub mod tz;
pub(crate) mod utils;
pub mod year_month;
pub mod zoneddatetime;

// TODO: evaluate positives and negatives of using tinystr.
// Re-exporting tinystr as a convenience, as it is currently tied into the API.
pub use tinystr::{TinyAsciiStr, TinyStr16, TinyStr4, TinyStr8};

pub use error::TemporalError;

/// The `Temporal` result type
pub type TemporalResult<T> = Result<T, TemporalError>;

// Relevant numeric constants
/// Nanoseconds per day constant: 8.64e+13
pub(crate) const NS_PER_DAY: i64 = 86_400_000_000_000;
/// Milliseconds per day constant: 8.64e+7
pub(crate) const MS_PER_DAY: i32 = 24 * 60 * 60 * 1000;
/// Max Instant nanosecond constant
pub(crate) const NS_MAX_INSTANT: i128 = NS_PER_DAY as i128 * 100_000_000i128;
/// Min Instant nanosecond constant
pub(crate) const NS_MIN_INSTANT: i128 = -NS_MAX_INSTANT;
