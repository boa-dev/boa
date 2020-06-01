use crate::syntax::{
    ast::{
        node::{Block, Node},
        Const,
    },
    parser::tests::check_parser,
};

#[test]
fn inline() {
    check_parser(
        "while (true) break;",
        vec![Node::while_loop(Const::from(true), Node::Break(None))],
    );
}

#[test]
fn new_line() {
    check_parser(
        "while (true)
            break;",
        vec![Node::while_loop(Const::from(true), Node::Break(None))],
    );
}

#[test]
fn inline_block_semicolon_insertion() {
    check_parser(
        "while (true) {break}",
        vec![Node::while_loop(
            Const::from(true),
            Block::from(vec![Node::Break(None)]),
        )],
    );
}

#[test]
fn new_line_semicolon_insertion() {
    check_parser(
        "while (true) {
            break test
        }",
        vec![Node::while_loop(
            Const::from(true),
            Block::from(vec![Node::break_node("test")]),
        )],
    );
}

#[test]
fn inline_block() {
    check_parser(
        "while (true) {break;}",
        vec![Node::while_loop(
            Const::from(true),
            Block::from(vec![Node::Break(None)]),
        )],
    );
}

#[test]
fn new_line_block() {
    check_parser(
        "while (true) {
            break test;
        }",
        vec![Node::while_loop(
            Const::from(true),
            Block::from(vec![Node::break_node("test")]),
        )],
    );
}

#[test]
fn reserved_label() {
    check_parser(
        "while (true) {
            break await;
        }",
        vec![Node::while_loop(
            Const::from(true),
            Block::from(vec![Node::break_node("await")]),
        )],
    );

    check_parser(
        "while (true) {
            break yield;
        }",
        vec![Node::while_loop(
            Const::from(true),
            Block::from(vec![Node::break_node("yield")]),
        )],
    );
}

#[test]
fn new_line_block_empty() {
    check_parser(
        "while (true) {
            break;
        }",
        vec![Node::while_loop(
            Const::from(true),
            Block::from(vec![Node::Break(None)]),
        )],
    );
}

#[test]
fn new_line_block_empty_semicolon_insertion() {
    check_parser(
        "while (true) {
            break
        }",
        vec![Node::while_loop(
            Const::from(true),
            Block::from(vec![Node::Break(None)]),
        )],
    );
}
