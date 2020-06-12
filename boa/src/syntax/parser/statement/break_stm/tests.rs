use crate::syntax::{
    ast::{
        node::{Block, Break, Node, WhileLoop},
        Const,
    },
    parser::tests::check_parser,
};

#[test]
fn inline() {
    check_parser(
        "while (true) break;",
        vec![WhileLoop::new(
            Const::from(true),
            Node::Break(Break::new::<_, Box<str>>(None)),
        )
        .into()],
    );
}

#[test]
fn new_line() {
    check_parser(
        "while (true)
            break;",
        vec![WhileLoop::new(Const::from(true), Break::new::<_, Box<str>>(None)).into()],
    );
}

#[test]
fn inline_block_semicolon_insertion() {
    check_parser(
        "while (true) {break}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new::<_, Box<str>>(None).into()]),
        )
        .into()],
    );
}

#[test]
fn new_line_semicolon_insertion() {
    check_parser(
        "while (true) {
            break test
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new("test").into()]),
        )
        .into()],
    );
}

#[test]
fn inline_block() {
    check_parser(
        "while (true) {break;}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new::<_, Box<str>>(None).into()]),
        )
        .into()],
    );
}

#[test]
fn new_line_block() {
    check_parser(
        "while (true) {
            break test;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new("test").into()]),
        )
        .into()],
    );
}

#[test]
fn reserved_label() {
    check_parser(
        "while (true) {
            break await;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new("await").into()]),
        )
        .into()],
    );

    check_parser(
        "while (true) {
            break yield;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new("yield").into()]),
        )
        .into()],
    );
}

#[test]
fn new_line_block_empty() {
    check_parser(
        "while (true) {
            break;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new::<_, Box<str>>(None).into()]),
        )
        .into()],
    );
}

#[test]
fn new_line_block_empty_semicolon_insertion() {
    check_parser(
        "while (true) {
            break
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new::<_, Box<str>>(None).into()]),
        )
        .into()],
    );
}
