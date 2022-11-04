//! This library implements the extension to query if a char belongs to a particular unicode identifier property.
//! Version: Unicode 15.0.0
//!
//! More information:
//!  - [Unicode® Standard Annex #31][uax31]
//!
//! [uax31]: http://unicode.org/reports/tr31

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
    clippy::let_unit_value
)]

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
