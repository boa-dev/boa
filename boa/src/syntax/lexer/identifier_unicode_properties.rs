//! This module implements the extension to query if a char belongs to a particular unicode identifier property.
//!
//! Unicode version: 13.0.0
//!
//! More information:
//!  - [Unicode® Standard Annex #31][uax31]
//!
//! [uax31]: http://unicode.org/reports/tr31

use super::identifier_unicode_tables;
use unicode_general_category::{get_general_category, GeneralCategory};
pub trait IdentifierUnicodeProperties: Sized + Copy {
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

impl IdentifierUnicodeProperties for char {
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
        table_binary_search(self, identifier_unicode_tables::OTHER_ID_START)
    }
    #[inline]
    fn is_other_id_continue(self) -> bool {
        table_binary_search(self, identifier_unicode_tables::OTHER_ID_CONTINUE)
    }
    #[inline]
    fn is_pattern_syntax(self) -> bool {
        table_binary_search(self, identifier_unicode_tables::PATTERN_SYNTAX)
    }
    #[inline]
    fn is_pattern_whitespace(self) -> bool {
        table_binary_search(self, identifier_unicode_tables::PATTERN_WHITE_SPACE)
    }
}
