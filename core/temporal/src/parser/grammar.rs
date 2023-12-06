//! ISO8601 specific grammar checks.

/// Checks if char is a `AKeyLeadingChar`.
#[inline]
pub(crate) const fn is_a_key_leading_char(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch == '_'
}

/// Checks if char is an `AKeyChar`.
#[inline]
pub(crate) const fn is_a_key_char(ch: char) -> bool {
    is_a_key_leading_char(ch) || ch.is_ascii_digit() || ch == '-'
}

/// Checks if char is an `AnnotationValueComponent`.
pub(crate) const fn is_annotation_value_component(ch: char) -> bool {
    ch.is_ascii_digit() || ch.is_ascii_alphabetic()
}

/// Checks if char is a `TZLeadingChar`.
#[inline]
pub(crate) const fn is_tz_leading_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '.'
}

/// Checks if char is a `TZChar`.
#[inline]
pub(crate) const fn is_tz_char(ch: char) -> bool {
    is_tz_leading_char(ch) || ch.is_ascii_digit() || ch == '-' || ch == '+'
}

/// Checks if char is a `TimeZoneIANAName` Separator.
pub(crate) const fn is_tz_name_separator(ch: char) -> bool {
    ch == '/'
}

/// Checks if char is an ascii sign.
pub(crate) const fn is_ascii_sign(ch: char) -> bool {
    ch == '+' || ch == '-'
}

/// Checks if char is an ascii sign or U+2212
pub(crate) const fn is_sign(ch: char) -> bool {
    is_ascii_sign(ch) || ch == '\u{2212}'
}

/// Checks if char is a `TimeSeparator`.
pub(crate) const fn is_time_separator(ch: char) -> bool {
    ch == ':'
}

/// Checks if char is a `TimeDesignator`.
pub(crate) const fn is_time_designator(ch: char) -> bool {
    ch == 'T' || ch == 't'
}

/// Checks if char is a `DateTimeSeparator`.
pub(crate) const fn is_date_time_separator(ch: char) -> bool {
    is_time_designator(ch) || ch == '\u{0020}'
}

/// Checks if char is a `UtcDesignator`.
pub(crate) const fn is_utc_designator(ch: char) -> bool {
    ch == 'Z' || ch == 'z'
}

/// Checks if char is a `DurationDesignator`.
pub(crate) const fn is_duration_designator(ch: char) -> bool {
    ch == 'P' || ch == 'p'
}

/// Checks if char is a `YearDesignator`.
pub(crate) const fn is_year_designator(ch: char) -> bool {
    ch == 'Y' || ch == 'y'
}

/// Checks if char is a `MonthsDesignator`.
pub(crate) const fn is_month_designator(ch: char) -> bool {
    ch == 'M' || ch == 'm'
}

/// Checks if char is a `WeekDesignator`.
pub(crate) const fn is_week_designator(ch: char) -> bool {
    ch == 'W' || ch == 'w'
}

/// Checks if char is a `DayDesignator`.
pub(crate) const fn is_day_designator(ch: char) -> bool {
    ch == 'D' || ch == 'd'
}

/// checks if char is a `DayDesignator`.
pub(crate) const fn is_hour_designator(ch: char) -> bool {
    ch == 'H' || ch == 'h'
}

/// Checks if char is a `MinuteDesignator`.
pub(crate) const fn is_minute_designator(ch: char) -> bool {
    is_month_designator(ch)
}

/// checks if char is a `SecondDesignator`.
pub(crate) const fn is_second_designator(ch: char) -> bool {
    ch == 'S' || ch == 's'
}

/// Checks if char is a `DecimalSeparator`.
pub(crate) const fn is_decimal_separator(ch: char) -> bool {
    ch == '.' || ch == ','
}

/// Checks if char is an `AnnotationOpen`.
pub(crate) const fn is_annotation_open(ch: char) -> bool {
    ch == '['
}

/// Checks if char is an `AnnotationClose`.
pub(crate) const fn is_annotation_close(ch: char) -> bool {
    ch == ']'
}

/// Checks if char is an `CriticalFlag`.
pub(crate) const fn is_critical_flag(ch: char) -> bool {
    ch == '!'
}

/// Checks if char is the `AnnotationKeyValueSeparator`.
pub(crate) const fn is_annotation_key_value_separator(ch: char) -> bool {
    ch == '='
}

/// Checks if char is a hyphen. Hyphens are used as a Date separator
/// and as a `AttributeValueComponent` separator.
pub(crate) const fn is_hyphen(ch: char) -> bool {
    ch == '-'
}
