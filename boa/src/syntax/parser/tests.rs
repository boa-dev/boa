//! Tests for the parser.

use super::Parser;
use crate::syntax::{ast::node::Node, ast::op::NumOp, lexer::Lexer};

#[allow(clippy::result_unwrap_used)]
pub(super) fn check_parser(js: &str, expr: &[Node]) {
    let mut lexer = Lexer::new(js);
    lexer.lex().expect("failed to lex");

    assert_eq!(
        Parser::new(&lexer.tokens)
            .parse_all()
            .expect("failed to parse"),
        Node::statement_list(expr)
    );
}

pub(super) fn check_invalid(js: &str) {
    let mut lexer = Lexer::new(js);
    lexer.lex().expect("failed to lex");

    assert!(Parser::new(&lexer.tokens).parse_all().is_err());
}

/// Should be parsed as `new Class().method()` instead of `new (Class().method())`
#[test]
fn check_construct_call_precedence() {
    check_parser(
        "new Date().getTime()",
        &[Node::call(
            Node::get_const_field(
                Node::new(Node::call(Node::local("Date"), Vec::new())),
                "getTime",
            ),
            Vec::new(),
        )],
    );
}

#[test]
fn assign_operator_precedence() {
    check_parser(
        "a = a + 1",
        &[Node::assign(
            Node::local("a"),
            Node::bin_op(NumOp::Add, Node::local("a"), Node::const_node(1.0)),
        )],
    );
}