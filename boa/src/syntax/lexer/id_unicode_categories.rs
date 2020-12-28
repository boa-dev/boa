use super::id_unicode_tables;
use unicode_general_category::{get_general_category, GeneralCategory};
pub trait IdentifierUnicodeCategories: Sized + Copy {
    fn is_id_start(self) -> bool;
    fn is_id_continue(self) -> bool;

    fn is_other_id_start(self) -> bool;
    fn is_other_id_continue(self) -> bool;
    fn is_pattern_syntax(self) -> bool;
    fn is_pattern_whitespace(self) -> bool;
}

fn table_binary_search(target: char, table: &'static [char]) -> bool {
    table.binary_search(&target).is_ok()
}

impl IdentifierUnicodeCategories for char {
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
        table_binary_search(self, id_unicode_tables::OTHER_ID_START)
    }
    #[inline]
    fn is_other_id_continue(self) -> bool {
        table_binary_search(self, id_unicode_tables::OTHER_ID_CONTINUE)
    }
    #[inline]
    fn is_pattern_syntax(self) -> bool {
        table_binary_search(self, id_unicode_tables::PATTERN_SYNTAX)
    }
    #[inline]
    fn is_pattern_whitespace(self) -> bool {
        table_binary_search(self, id_unicode_tables::PATTERN_WHITE_SPACE)
    }
}
