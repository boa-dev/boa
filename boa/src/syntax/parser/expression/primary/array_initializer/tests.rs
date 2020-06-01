// ! Tests for array initializer parsing.

use crate::syntax::{
    ast::{node::ArrayDecl, Const},
    parser::tests::check_parser,
};

/// Checks an empty array.
#[test]
fn check_empty() {
    check_parser("[]", vec![ArrayDecl::from(vec![]).into()]);
}

/// Checks an array with empty slot.
#[test]
fn check_empty_slot() {
    check_parser(
        "[,]",
        vec![ArrayDecl::from(vec![Const::Undefined.into()]).into()],
    );
}

/// Checks a numeric array.
#[test]
fn check_numeric_array() {
    check_parser(
        "[1, 2, 3]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from(2).into(),
            Const::from(3).into(),
        ])
        .into()],
    );
}

// Checks a numeric array with trailing comma
#[test]
fn check_numeric_array_trailing() {
    check_parser(
        "[1, 2, 3,]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from(2).into(),
            Const::from(3).into(),
        ])
        .into()],
    );
}

/// Checks a numeric array with an elision.
#[test]
fn check_numeric_array_elision() {
    check_parser(
        "[1, 2, , 3]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from(2).into(),
            Const::Undefined.into(),
            Const::from(3).into(),
        ])
        .into()],
    );
}

/// Checks a numeric array with repeated elisions.
#[test]
fn check_numeric_array_repeated_elision() {
    check_parser(
        "[1, 2, ,, 3]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from(2).into(),
            Const::Undefined.into(),
            Const::Undefined.into(),
            Const::from(3).into(),
        ])
        .into()],
    );
}

/// Checks a combined array.
#[test]
fn check_combined() {
    check_parser(
        "[1, \"a\", 2]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from("a").into(),
            Const::from(2).into(),
        ])
        .into()],
    );
}

/// Checks a combined array with an empty string
#[test]
fn check_combined_empty_str() {
    check_parser(
        "[1, \"\", 2]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from("").into(),
            Const::from(2).into(),
        ])
        .into()],
    );
}
