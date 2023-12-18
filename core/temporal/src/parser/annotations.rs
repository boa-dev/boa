/// Parsing for Temporal's `Annotations`.
use crate::{
    assert_syntax,
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
        return Err(
            TemporalError::syntax().with_message("ZonedDateTime must have a TimeZone annotation.")
        );
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
    assert_syntax!(
        is_annotation_open(cursor.abrupt_next()?),
        "Invalid annotation open character."
    );

    let critical = cursor.check_or(false, is_critical_flag);
    cursor.advance_if(critical);

    // Parse AnnotationKey.
    let annotation_key = parse_annotation_key(cursor)?;
    assert_syntax!(
        is_annotation_key_value_separator(cursor.abrupt_next()?),
        "Invalid annotation key-value separator"
    );

    // Parse AnnotationValue.
    let annotation_value = parse_annotation_value(cursor)?;
    assert_syntax!(
        is_annotation_close(cursor.abrupt_next()?),
        "Invalid annotion closing character"
    );

    Ok(KeyValueAnnotation {
        key: annotation_key,
        value: annotation_value,
        critical,
    })
}

/// Parse an `AnnotationKey`.
fn parse_annotation_key(cursor: &mut Cursor) -> TemporalResult<String> {
    let key_start = cursor.pos();
    assert_syntax!(
        is_a_key_leading_char(cursor.abrupt_next()?),
        "Invalid key leading character."
    );

    while let Some(potential_key_char) = cursor.next() {
        // End of key.
        if cursor.check_or(false, is_annotation_key_value_separator) {
            // Return found key
            return Ok(cursor.slice(key_start, cursor.pos()));
        }

        assert_syntax!(
            is_a_key_char(potential_key_char),
            "Invalid annotation key character."
        );
    }

    Err(TemporalError::abrupt_end())
}

/// Parse an `AnnotationValue`.
fn parse_annotation_value(cursor: &mut Cursor) -> TemporalResult<String> {
    let value_start = cursor.pos();
    cursor.advance();
    while let Some(potential_value_char) = cursor.next() {
        if cursor.check_or(false, is_annotation_close) {
            // Return the determined AnnotationValue.
            return Ok(cursor.slice(value_start, cursor.pos()));
        }

        if is_hyphen(potential_value_char) {
            assert_syntax!(
                cursor.peek().map_or(false, is_annotation_value_component),
                "Missing annotation value compoenent after '-'"
            );
            cursor.advance();
            continue;
        }

        assert_syntax!(
            is_annotation_value_component(potential_value_char),
            "Invalid annotation value component character."
        );
    }

    Err(TemporalError::abrupt_end())
}
