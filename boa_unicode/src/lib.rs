//! This crate implements the extension to query if a char belongs to a particular unicode identifier property.
//! Version: Unicode 13.0.0
//!
//! More information:
//!  - [Unicode® Standard Annex #31][uax31]
//!
//! [uax31]: http://unicode.org/reports/tr31

mod tables;

use unicode_general_category::{get_general_category, GeneralCategory};

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
    /// Returns `true` if this value is a member of "ID_Start".
    fn is_id_start(self) -> bool;

    /// Returns `true` if this value is a member of "ID_Continue".
    fn is_id_continue(self) -> bool;

    /// Returns `true` if this value is a member of "Other_ID_Start".
    fn is_other_id_start(self) -> bool;

    /// Returns `true` if this value is a member of "Other_ID_Continue".
    fn is_other_id_continue(self) -> bool;

    /// Returns `true` if this value is a member of "Pattern_Syntax".
    fn is_pattern_syntax(self) -> bool;

    /// Returns `true` if this value is a member of "Pattern_White_Space".
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
