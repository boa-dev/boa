use crate::syntax::{ast::node::Node, parser::tests::check_parser};

#[test]
fn check_inline() {
    check_parser(
        "while (true) break;",
        &[Node::while_loop(Node::const_node(true), Node::Break(None))],
    );
}

#[test]
fn check_new_line() {
    check_parser(
        "while (true)
            break;",
        &[Node::while_loop(Node::const_node(true), Node::Break(None))],
    );
}

#[test]
fn check_inline_block_semicolon_insertion() {
    check_parser(
        "while (true) {break}",
        &[Node::while_loop(
            Node::const_node(true),
            Node::Block(vec![Node::Break(None)]),
        )],
    );
}

#[test]
fn check_new_line_semicolon_insertion() {
    check_parser(
        "while (true) {
            break test
        }",
        &[Node::while_loop(
            Node::const_node(true),
            Node::Block(vec![Node::break_node("test")]),
        )],
    );
}

#[test]
fn check_inline_block() {
    check_parser(
        "while (true) {break;}",
        &[Node::while_loop(
            Node::const_node(true),
            Node::Block(vec![Node::Break(None)]),
        )],
    );
}

#[test]
fn check_new_line_block() {
    check_parser(
        "while (true) {
            break test;
        }",
        &[Node::while_loop(
            Node::const_node(true),
            Node::Block(vec![Node::break_node("test")]),
        )],
    );
}

#[test]
fn check_new_line_block_empty() {
    check_parser(
        "while (true) {
            break;
        }",
        &[Node::while_loop(
            Node::const_node(true),
            Node::Block(vec![Node::Break(None)]),
        )],
    );
}

#[test]
fn check_new_line_block_empty_semicolon_insertion() {
    check_parser(
        "while (true) {
            break
        }",
        &[Node::while_loop(
            Node::const_node(true),
            Node::Block(vec![Node::Break(None)]),
        )],
    );
}
