/// Parsing for Temporal's `Annotations`.

use crate::{
    error::{Error, ParseResult},
    lexer::Error as LexError,
    temporal::{IsoCursor, grammar::*}
};

use boa_ast::{
    Position,
    temporal::KeyValueAnnotation,
};

use rustc_hash::FxHashMap;

/// Parse any number of `KeyValueAnnotation`s
pub(crate) fn parse_annotations(cursor: &mut IsoCursor) -> ParseResult<FxHashMap<String, (bool, String)>> {
    let mut hash_map = FxHashMap::default();
    while let Some(annotation_open) = cursor.peek() {
        if *annotation_open == '[' {
            let kv = parse_kv_annotation(cursor)?;
            if !hash_map.contains_key(&kv.key) {
                hash_map.insert(kv.key, (kv.critical, kv.value));
            }
        } else {
            break;
        }
    }

    return Ok(hash_map);
}

/// Parse an annotation with an `AnnotationKey`=`AnnotationValue` pair.
fn parse_kv_annotation(cursor: &mut IsoCursor) -> ParseResult<KeyValueAnnotation> {
    assert!(*cursor.peek().unwrap() == '[');
    // TODO: remove below if unneeded.
    let _start = Position::new(1, (cursor.pos() + 1) as u32);

    let potential_critical = cursor.next().ok_or_else(|| Error::AbruptEnd)?;
    let (leading_char, critical) = if *potential_critical == '!' {
        (cursor.next().ok_or_else(|| Error::AbruptEnd)?, true)
    } else {
        (potential_critical, false)
    };

    if !is_a_key_leading_char(leading_char) {
        return Err(LexError::syntax(
            "Invalid AnnotationKey leading character",
            Position::new(1, (cursor.pos() + 1) as u32),
        ).into());
    }

    // Parse AnnotationKey.
    let annotation_key = parse_annotation_key(cursor)?;

    // Advance past the '=' character.
    assert!(*cursor.peek().unwrap() == '=');
    cursor.advance();

    // Parse AnnotationValue.
    let annotation_value = parse_annotation_value(cursor)?;

    // Assert that we are at the annotation close and advance cursor past annotation to close.
    assert!(*cursor.peek().unwrap() == ']');
    // TODO: remove below if unneeded.
    let _end = Position::new(1, (cursor.pos() + 1) as u32);
    cursor.advance();

    return Ok(KeyValueAnnotation {
        key: annotation_key,
        value: annotation_value,
        critical,
    });
}

/// Parse an `AnnotationKey`.
fn parse_annotation_key(cursor: &mut IsoCursor) -> ParseResult<String> {
    let key_start = cursor.pos();
    while let Some(potential_key_char) = cursor.next() {
        // End of key.
        if *potential_key_char == '=' {
            // Return found key
            return Ok(cursor.slice(key_start, cursor.pos()));
        }

        if !is_a_key_char(potential_key_char) {
            return Err(LexError::syntax(
                "Invalid AnnotationKey Character",
                Position::new(1, (cursor.pos() + 1) as u32),
            ).into());
        }
    }

    Err(Error::AbruptEnd)
}

/// Parse an `AnnotationValue`.
fn parse_annotation_value(cursor: &mut IsoCursor) -> ParseResult<String> {
    let value_start = cursor.pos();
    while let Some(potential_value_char) = cursor.next() {
        if *potential_value_char == ']' {
            // Return the determined AnnotationValue.
            return Ok(cursor.slice(value_start, cursor.pos()));
        }

        if *potential_value_char == '-' {
            if !cursor
                .peek_n(1)
                .map(|ch| is_annotation_value_component(ch))
                .unwrap_or(false)
            {
                return Err(LexError::syntax(
                    "Missing AttributeValueComponent after '-'",
                    Position::new(1, (cursor.pos() + 1) as u32),
                ).into());
            }
            cursor.advance();
            continue;
        }

        if !is_annotation_value_component(potential_value_char) {
            return Err(LexError::syntax(
                "Invalid character in AnnotationValue",
                Position::new(1, (value_start + cursor.pos() + 1) as u32),
            ).into());
        }
    }

    Err(Error::AbruptEnd)
}