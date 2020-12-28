use super::id_unicode_tables;
use unicode_categories::UnicodeCategories;
pub trait IdentifierUnicodeCategories: Sized + Copy + UnicodeCategories {
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
            && (self.is_letter() || self.is_number_letter() || self.is_other_id_start())
    }

    #[inline]
    fn is_id_continue(self) -> bool {
        !self.is_pattern_syntax()
            && !self.is_pattern_whitespace()
            && (self.is_id_start()
                || self.is_mark_nonspacing()
                || self.is_mark_spacing_combining()
                || self.is_number_decimal_digit()
                || self.is_punctuation_connector()
                || self.is_other_id_continue())
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
