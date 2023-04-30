use super::*;

#[test]
fn check_unicode_version() {
    assert_eq!(UNICODE_VERSION, unicode_general_category::UNICODE_VERSION);
}

#[test]
fn ut_is_id_start() {
    // Sunny day
    for c in 'a'..='z' {
        assert!(c.is_id_start());
    }
    for c in 'A'..='Z' {
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
fn ut_is_id_continue() {
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
    assert!('_'.is_id_continue());

    // Rainy day
    assert!(!' '.is_id_continue());
    assert!(!'\n'.is_id_continue());
    assert!(!'\t'.is_id_continue());
    assert!(!'!'.is_id_continue());
    assert!(!';'.is_id_continue());
    assert!(!'-'.is_id_continue());
    assert!(!'='.is_id_continue());
    assert!(!'+'.is_id_continue());
    assert!(!'('.is_id_continue());
    assert!(!'('.is_id_continue());
}

#[test]
fn ut_is_orther_id_start() {
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
fn ut_is_orther_id_continue() {
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
fn ut_is_pattern_syntax() {
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
fn ut_is_pattern_whitespace() {
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
