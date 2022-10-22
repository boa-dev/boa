// ! Tests for array initializer parsing.

use crate::syntax::{
    ast::{
        expression::literal::{ArrayLiteral, Literal},
        Expression, Statement,
    },
    parser::tests::check_parser,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;

/// Checks an empty array.
#[test]
fn check_empty() {
    check_parser(
        "[]",
        vec![Statement::Expression(Expression::from(ArrayLiteral::from(vec![]))).into()],
        Interner::default(),
    );
}

/// Checks an array with empty slot.
#[test]
fn check_empty_slot() {
    check_parser(
        "[,]",
        vec![Statement::Expression(Expression::from(ArrayLiteral::from(vec![None]))).into()],
        Interner::default(),
    );
}

/// Checks a numeric array.
#[test]
fn check_numeric_array() {
    check_parser(
        "[1, 2, 3]",
        vec![
            Statement::Expression(Expression::from(ArrayLiteral::from(vec![
                Some(Literal::from(1).into()),
                Some(Literal::from(2).into()),
                Some(Literal::from(3).into()),
            ])))
            .into(),
        ],
        Interner::default(),
    );
}

// Checks a numeric array with trailing comma
#[test]
fn check_numeric_array_trailing() {
    check_parser(
        "[1, 2, 3,]",
        vec![
            Statement::Expression(Expression::from(ArrayLiteral::from(vec![
                Some(Literal::from(1).into()),
                Some(Literal::from(2).into()),
                Some(Literal::from(3).into()),
            ])))
            .into(),
        ],
        Interner::default(),
    );
}

/// Checks a numeric array with an elision.
#[test]
fn check_numeric_array_elision() {
    check_parser(
        "[1, 2, , 3]",
        vec![
            Statement::Expression(Expression::from(ArrayLiteral::from(vec![
                Some(Literal::from(1).into()),
                Some(Literal::from(2).into()),
                None,
                Some(Literal::from(3).into()),
            ])))
            .into(),
        ],
        Interner::default(),
    );
}

/// Checks a numeric array with repeated elisions.
#[test]
fn check_numeric_array_repeated_elision() {
    check_parser(
        "[1, 2, ,, 3]",
        vec![
            Statement::Expression(Expression::from(ArrayLiteral::from(vec![
                Some(Literal::from(1).into()),
                Some(Literal::from(2).into()),
                None,
                None,
                Some(Literal::from(3).into()),
            ])))
            .into(),
        ],
        Interner::default(),
    );
}

/// Checks a combined array.
#[test]
fn check_combined() {
    let mut interner = Interner::default();
    check_parser(
        "[1, \"a\", 2]",
        vec![
            Statement::Expression(Expression::from(ArrayLiteral::from(vec![
                Some(Literal::from(1).into()),
                Some(Literal::from(interner.get_or_intern_static("a", utf16!("a"))).into()),
                Some(Literal::from(2).into()),
            ])))
            .into(),
        ],
        interner,
    );
}

/// Checks a combined array with an empty string
#[test]
fn check_combined_empty_str() {
    check_parser(
        "[1, \"\", 2]",
        vec![
            Statement::Expression(Expression::from(ArrayLiteral::from(vec![
                Some(Literal::from(1).into()),
                Some(Literal::from(Sym::EMPTY_STRING).into()),
                Some(Literal::from(2).into()),
            ])))
            .into(),
        ],
        Interner::default(),
    );
}
