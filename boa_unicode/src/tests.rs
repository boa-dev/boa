#![allow(clippy::cognitive_complexity)]

use super::*;

#[test]
fn check_unicode_version() {
    assert_eq!(UNICODE_VERSION, unicode_general_category::UNICODE_VERSION);
}

#[test]
fn is_id_start() {
    // Sunny day
    for c in 'a'..='z' {
        assert!(c.is_id_start());
    }
    for c in 'A'..='Z' {
        assert!(c.is_id_start());
    }
    for c in '\u{00E0}'..='\u{00F6}' {
        assert!(c.is_id_start());
    }
    for c in '\u{02B0}'..='\u{02C1}' {
        assert!(c.is_id_start());
    }
    for c in '\u{05D0}'..='\u{05DE}' {
        assert!(c.is_id_start());
    }
    for c in '\u{1F88}'..='\u{1F89}' {
        assert!(c.is_id_start());
    }
    for c in '\u{0391}'..='\u{039F}' {
        assert!(c.is_id_start());
    }
    for c in '\u{2160}'..='\u{216F}' {
        assert!(c.is_id_start());
    }

    // Rainy day
    for c in '0'..='9' {
        assert!(!c.is_id_start());
    }
    assert!(!' '.is_id_start());
    assert!(!'\n'.is_id_start());
    assert!(!'\t'.is_id_start());
    assert!(!'!'.is_id_start());
    assert!(!';'.is_id_start());
    assert!(!'-'.is_id_start());
    assert!(!'_'.is_id_start());
    assert!(!'='.is_id_start());
    assert!(!'+'.is_id_start());
    assert!(!'('.is_id_start());
    assert!(!')'.is_id_start());
}

#[test]
fn is_id_continue() {
    // Sunny day
    for c in 'a'..='z' {
        assert!(c.is_id_continue());
    }
    for c in 'A'..='Z' {
        assert!(c.is_id_continue());
    }
    for c in '0'..='9' {
        assert!(c.is_id_continue());
    }
    for c in '\u{0300}'..='\u{036F}' {
        assert!(c.is_id_continue());
    }
    for c in '\u{093E}'..='\u{094F}' {
        assert!(c.is_id_continue());
    }
    for c in '\u{0660}'..='\u{0669}' {
        assert!(c.is_id_continue());
    }
    for c in ['_', '\u{203F}', '\u{2040}', '\u{2054}', '\u{FE33}'] {
        assert!(c.is_id_continue());
    }

    // Rainy day
    for c in [' ', '\n', '\t', '!', ';', '-', '=', '+', '(', '('] {
        assert!(!c.is_id_continue());
    }
}

#[test]
fn is_orther_id_start() {
    // Sunny day
    for c in tables::OTHER_ID_START {
        assert!(c.is_other_id_start());
    }

    // Rainy day
    for c in [' ', '\n', '='] {
        assert!(!c.is_other_id_start());
    }
}

#[test]
fn is_orther_id_continue() {
    // Sunny day
    for c in tables::OTHER_ID_CONTINUE {
        assert!(c.is_other_id_continue());
    }

    // Rainy day
    for c in [' ', '\n', '='] {
        assert!(!c.is_other_id_continue());
    }
}

#[test]
fn is_pattern_syntax() {
    // Sunny day
    for c in tables::PATTERN_SYNTAX {
        assert!(c.is_pattern_syntax());
    }

    // Rainy day
    for c in [' ', '\t', '\n', '\r'] {
        assert!(!c.is_pattern_syntax());
    }
}

#[test]
fn is_pattern_whitespace() {
    // Sunny day
    for c in tables::PATTERN_WHITE_SPACE {
        assert!(c.is_pattern_whitespace());
    }

    // Rainy day
    for c in ['+', '~', '`', '!', '@', '^', '='] {
        assert!(!c.is_pattern_whitespace());
    }
    for c in '0'..='9' {
        assert!(!c.is_pattern_whitespace());
    }
    for c in 'a'..='z' {
        assert!(!c.is_pattern_whitespace());
    }
    for c in 'A'..='Z' {
        assert!(!c.is_pattern_whitespace());
    }
}
