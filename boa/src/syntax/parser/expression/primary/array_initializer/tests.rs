// ! Tests for array initializer parsing.

use crate::syntax::{
    ast::{constant::Const, node::Node},
    parser::tests::check_parser,
};

/// Checks an empty array.
#[test]
fn check_empty() {
    check_parser("[]", vec![Node::array_decl(Vec::new())]);
}

/// Checks an array with empty slot.
#[test]
fn check_empty_slot() {
    check_parser(
        "[,]",
        vec![Node::array_decl(vec![Node::Const(Const::Undefined)])],
    );
}

/// Checks a numeric array.
#[test]
fn check_numeric_array() {
    check_parser(
        "[1, 2, 3]",
        vec![Node::array_decl(vec![
            Node::const_node(1),
            Node::const_node(2),
            Node::const_node(3),
        ])],
    );
}

// Checks a numeric array with trailing comma
#[test]
fn check_numeric_array_trailing() {
    check_parser(
        "[1, 2, 3,]",
        vec![Node::array_decl(vec![
            Node::const_node(1),
            Node::const_node(2),
            Node::const_node(3),
        ])],
    );
}

/// Checks a numeric array with an elision.
#[test]
fn check_numeric_array_elision() {
    check_parser(
        "[1, 2, , 3]",
        vec![Node::array_decl(vec![
            Node::const_node(1),
            Node::const_node(2),
            Node::Const(Const::Undefined),
            Node::const_node(3),
        ])],
    );
}

/// Checks a numeric array with repeated elisions.
#[test]
fn check_numeric_array_repeated_elision() {
    check_parser(
        "[1, 2, ,, 3]",
        vec![Node::array_decl(vec![
            Node::const_node(1),
            Node::const_node(2),
            Node::Const(Const::Undefined),
            Node::Const(Const::Undefined),
            Node::const_node(3),
        ])],
    );
}

/// Checks a combined array.
#[test]
fn check_combined() {
    check_parser(
        "[1, \"a\", 2]",
        vec![Node::array_decl(vec![
            Node::const_node(1),
            Node::const_node("a"),
            Node::const_node(2),
        ])],
    );
}

/// Checks a combined array with an empty string
#[test]
fn check_combined_empty_str() {
    check_parser(
        "[1, \"\", 2]",
        vec![Node::array_decl(vec![
            Node::const_node(1),
            Node::const_node(""),
            Node::const_node(2),
        ])],
    );
}
