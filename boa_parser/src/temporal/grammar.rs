//! ISO8601 specific grammar checks.

/// Checks if char is a `AKeyLeadingChar`.
#[inline]
pub(crate) fn is_a_key_leading_char(ch: &char) -> bool {
    ch.is_ascii_lowercase() || *ch == '_'
}

/// Checks if char is an `AKeyChar`.
#[inline]
pub(crate) fn is_a_key_char(ch: &char) -> bool {
    is_a_key_leading_char(ch) || ch.is_ascii_digit() || *ch == '-'
}

/// Checks if char is an `AnnotationValueComponent`.
pub(crate) fn is_annotation_value_component(ch: &char) -> bool {
    ch.is_ascii_digit() || ch.is_ascii_alphabetic()
}

/// Checks if char is a `TZLeadingChar`.
#[inline]
pub(crate) fn is_tz_leading_char(ch: &char) -> bool {
    ch.is_ascii_alphabetic() || *ch == '_' || *ch == '.'
}

/// Checks if char is a `TZChar`.
#[inline]
pub(crate) fn is_tz_char(ch: &char) -> bool {
    is_tz_leading_char(ch) || ch.is_ascii_digit() || *ch == '-' || *ch == '+'
}

/// Checks if char is an ascii sign.
pub(crate) fn is_ascii_sign(ch: &char) -> bool {
    *ch == '+' || *ch == '-'
}

/// Checks if char is an ascii sign or U+2212
pub(crate) fn is_sign(ch: &char) -> bool {
    is_ascii_sign(ch) || *ch == '\u{2212}'
}

/// Checks if char is a `DateTimeSeparator`.
pub(crate) fn is_date_time_separator(ch: &char) -> bool {
    *ch == 'T' || *ch == 't' || *ch == '\u{0020}'
}

/// Checks if char is a `UtcDesignator`.
pub(crate) fn is_utc_designator(ch: &char) -> bool {
    *ch == 'Z' || *ch == 'z'
}

/// Checks if char is a `DecimalSeparator`.
pub(crate) fn is_decimal_separator(ch: &char) -> bool {
    *ch == '.' || *ch == ','
}