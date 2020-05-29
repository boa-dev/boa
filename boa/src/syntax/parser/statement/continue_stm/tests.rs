use crate::syntax::{
    ast::{
        node::{Block, Node, WhileLoop},
        Const,
    },
    parser::tests::check_parser,
};

#[test]
fn inline() {
    check_parser(
        "while (true) continue;",
        vec![WhileLoop::new(Const::from(true), Node::Continue(None)).into()],
    );
}

#[test]
fn new_line() {
    check_parser(
        "while (true)
            continue;",
        vec![WhileLoop::new(Const::from(true), Node::Continue(None)).into()],
    );
}

#[test]
fn inline_block_semicolon_insertion() {
    check_parser(
        "while (true) {continue}",
        vec![WhileLoop::new(Const::from(true), Block::from(vec![Node::Continue(None)])).into()],
    );
}

#[test]
fn new_line_semicolon_insertion() {
    check_parser(
        "while (true) {
            continue test
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Node::continue_node("test")]),
        )
        .into()],
    );
}

#[test]
fn inline_block() {
    check_parser(
        "while (true) {continue;}",
        vec![WhileLoop::new(Const::from(true), Block::from(vec![Node::Continue(None)])).into()],
    );
}

#[test]
fn new_line_block() {
    check_parser(
        "while (true) {
            continue test;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Node::continue_node("test")]),
        )
        .into()],
    );
}

#[test]
fn reserved_label() {
    check_parser(
        "while (true) {
            continue await;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Node::continue_node("await")]),
        )
        .into()],
    );

    check_parser(
        "while (true) {
            continue yield;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Node::continue_node("yield")]),
        )
        .into()],
    );
}

#[test]
fn new_line_block_empty() {
    check_parser(
        "while (true) {
            continue;
        }",
        vec![WhileLoop::new(Const::from(true), Block::from(vec![Node::Continue(None)])).into()],
    );
}

#[test]
fn new_line_block_empty_semicolon_insertion() {
    check_parser(
        "while (true) {
            continue
        }",
        vec![WhileLoop::new(Const::from(true), Block::from(vec![Node::Continue(None)])).into()],
    );
}
