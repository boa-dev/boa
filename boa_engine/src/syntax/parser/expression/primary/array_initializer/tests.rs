// ! Tests for array initializer parsing.

use crate::syntax::{
    ast::{node::ArrayDecl, Const, Node},
    parser::tests::check_parser,
};
use boa_interner::{Interner, Sym};

/// Checks an empty array.
#[test]
fn check_empty() {
    let mut interner = Interner::default();
    check_parser("[]", vec![ArrayDecl::from(vec![]).into()], &mut interner);
}

/// Checks an array with empty slot.
#[test]
fn check_empty_slot() {
    let mut interner = Interner::default();
    check_parser(
        "[,]",
        vec![ArrayDecl::from(vec![Node::Empty]).into()],
        &mut interner,
    );
}

/// Checks a numeric array.
#[test]
fn check_numeric_array() {
    let mut interner = Interner::default();
    check_parser(
        "[1, 2, 3]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from(2).into(),
            Const::from(3).into(),
        ])
        .into()],
        &mut interner,
    );
}

// Checks a numeric array with trailing comma
#[test]
fn check_numeric_array_trailing() {
    let mut interner = Interner::default();
    check_parser(
        "[1, 2, 3,]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from(2).into(),
            Const::from(3).into(),
        ])
        .into()],
        &mut interner,
    );
}

/// Checks a numeric array with an elision.
#[test]
fn check_numeric_array_elision() {
    let mut interner = Interner::default();
    check_parser(
        "[1, 2, , 3]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from(2).into(),
            Node::Empty,
            Const::from(3).into(),
        ])
        .into()],
        &mut interner,
    );
}

/// Checks a numeric array with repeated elisions.
#[test]
fn check_numeric_array_repeated_elision() {
    let mut interner = Interner::default();
    check_parser(
        "[1, 2, ,, 3]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from(2).into(),
            Node::Empty,
            Node::Empty,
            Const::from(3).into(),
        ])
        .into()],
        &mut interner,
    );
}

/// Checks a combined array.
#[test]
fn check_combined() {
    let mut interner = Interner::default();
    check_parser(
        "[1, \"a\", 2]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from(interner.get_or_intern_static("a")).into(),
            Const::from(2).into(),
        ])
        .into()],
        &mut interner,
    );
}

/// Checks a combined array with an empty string
#[test]
fn check_combined_empty_str() {
    let mut interner = Interner::default();
    check_parser(
        "[1, \"\", 2]",
        vec![ArrayDecl::from(vec![
            Const::from(1).into(),
            Const::from(Sym::EMPTY_STRING).into(),
            Const::from(2).into(),
        ])
        .into()],
        &mut interner,
    );
}
