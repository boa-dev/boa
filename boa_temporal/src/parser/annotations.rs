/// Parsing for Temporal's `Annotations`.
use crate::{
    parser::{
        grammar::{
            is_a_key_char, is_a_key_leading_char, is_annotation_close,
            is_annotation_key_value_separator, is_annotation_value_component, is_critical_flag,
        },
        time_zone,
        time_zone::TimeZoneAnnotation,
        Cursor,
    },
    TemporalError, TemporalResult,
};

use super::grammar::{is_annotation_open, is_hyphen};

/// A `KeyValueAnnotation` Parse Node.
#[derive(Debug, Clone)]
pub(crate) struct KeyValueAnnotation {
    /// An `Annotation`'s Key.
    pub(crate) key: String,
    /// An `Annotation`'s value.
    pub(crate) value: String,
    /// Whether the annotation was flagged as critical.
    pub(crate) critical: bool,
}

/// Strictly a Parsing Intermediary for the checking the common annotation backing.
pub(crate) struct AnnotationSet {
    pub(crate) tz: Option<TimeZoneAnnotation>,
    pub(crate) calendar: Option<String>,
}

/// Parse a `TimeZoneAnnotation` `Annotations` set
pub(crate) fn parse_annotation_set(
    zoned: bool,
    cursor: &mut Cursor,
) -> TemporalResult<AnnotationSet> {
    // Parse the first annotation.
    let tz_annotation = time_zone::parse_ambiguous_tz_annotation(cursor)?;

    if tz_annotation.is_none() && zoned {
        return Err(TemporalError::syntax()
            .with_message("iso8601 ZonedDateTime requires a TimeZoneAnnotation."));
    }

    // Parse any `Annotations`
    let annotations = cursor.check_or(false, is_annotation_open);

    if annotations {
        let annotations = parse_annotations(cursor)?;
        return Ok(AnnotationSet {
            tz: tz_annotation,
            calendar: annotations.calendar,
        });
    }

    Ok(AnnotationSet {
        tz: tz_annotation,
        calendar: None,
    })
}

/// An internal crate type to house any recognized annotations that are found.
#[derive(Default)]
pub(crate) struct RecognizedAnnotations {
    pub(crate) calendar: Option<String>,
}

/// Parse any number of `KeyValueAnnotation`s
pub(crate) fn parse_annotations(cursor: &mut Cursor) -> TemporalResult<RecognizedAnnotations> {
    let mut annotations = RecognizedAnnotations::default();

    let mut calendar_crit = false;
    while cursor.check_or(false, is_annotation_open) {
        let kv = parse_kv_annotation(cursor)?;

        if &kv.key == "u-ca" {
            if annotations.calendar.is_none() {
                annotations.calendar = Some(kv.value);
                calendar_crit = kv.critical;
                continue;
            }

            if calendar_crit || kv.critical {
                return Err(TemporalError::syntax().with_message(
                    "Cannot have critical flag with duplicate calendar annotations",
                ));
            }
        } else if kv.critical {
            return Err(TemporalError::syntax().with_message("Unrecognized critical annotation."));
        }
    }

    Ok(annotations)
}

/// Parse an annotation with an `AnnotationKey`=`AnnotationValue` pair.
fn parse_kv_annotation(cursor: &mut Cursor) -> TemporalResult<KeyValueAnnotation> {
    debug_assert!(cursor.check_or(false, is_annotation_open));

    let potential_critical = cursor.next().ok_or_else(TemporalError::abrupt_end)?;
    let (leading_char, critical) = if is_critical_flag(potential_critical) {
        (cursor.next().ok_or_else(TemporalError::abrupt_end)?, true)
    } else {
        (potential_critical, false)
    };

    if !is_a_key_leading_char(leading_char) {
        return Err(TemporalError::syntax().with_message("Invalid AnnotationKey leading character"));
    }

    // Parse AnnotationKey.
    let annotation_key = parse_annotation_key(cursor)?;

    debug_assert!(cursor.check_or(false, is_annotation_key_value_separator));
    // Advance past the '=' character.
    cursor.advance();

    // Parse AnnotationValue.
    let annotation_value = parse_annotation_value(cursor)?;

    // Assert that we are at the annotation close and advance cursor past annotation to close.
    debug_assert!(cursor.check_or(false, is_annotation_close));
    cursor.advance();

    Ok(KeyValueAnnotation {
        key: annotation_key,
        value: annotation_value,
        critical,
    })
}

/// Parse an `AnnotationKey`.
fn parse_annotation_key(cursor: &mut Cursor) -> TemporalResult<String> {
    let key_start = cursor.pos();
    while let Some(potential_key_char) = cursor.next() {
        // End of key.
        if is_annotation_key_value_separator(potential_key_char) {
            // Return found key
            return Ok(cursor.slice(key_start, cursor.pos()));
        }

        if !is_a_key_char(potential_key_char) {
            return Err(TemporalError::syntax().with_message("Invalid AnnotationKey Character"));
        }
    }

    Err(TemporalError::abrupt_end())
}

/// Parse an `AnnotationValue`.
fn parse_annotation_value(cursor: &mut Cursor) -> TemporalResult<String> {
    let value_start = cursor.pos();
    while let Some(potential_value_char) = cursor.next() {
        if is_annotation_close(potential_value_char) {
            // Return the determined AnnotationValue.
            return Ok(cursor.slice(value_start, cursor.pos()));
        }

        if is_hyphen(potential_value_char) {
            if !cursor
                .peek_n(1)
                .map_or(false, is_annotation_value_component)
            {
                return Err(TemporalError::syntax()
                    .with_message("Missing AttributeValueComponent after '-'"));
            }
            cursor.advance();
            continue;
        }

        if !is_annotation_value_component(potential_value_char) {
            return Err(
                TemporalError::syntax().with_message("Invalid character in AnnotationValue")
            );
        }
    }

    Err(TemporalError::abrupt_end())
}
