use crate::syntax::{
    ast::{
        node::{Block, Continue, WhileLoop},
        Const,
    },
    parser::tests::check_parser,
};

#[test]
fn inline() {
    check_parser(
        "while (true) continue;",
        vec![WhileLoop::new(Const::from(true), Continue::new::<_, Box<str>>(None)).into()],
    );
}

#[test]
fn new_line() {
    check_parser(
        "while (true)
            continue;",
        vec![WhileLoop::new(Const::from(true), Continue::new::<_, Box<str>>(None)).into()],
    );
}

#[test]
fn inline_block_semicolon_insertion() {
    check_parser(
        "while (true) {continue}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new::<_, Box<str>>(None).into()]),
        )
        .into()],
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
            Block::from(vec![Continue::new("test").into()]),
        )
        .into()],
    );
}

#[test]
fn inline_block() {
    check_parser(
        "while (true) {continue;}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new::<_, Box<str>>(None).into()]),
        )
        .into()],
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
            Block::from(vec![Continue::new("test").into()]),
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
            Block::from(vec![Continue::new("await").into()]),
        )
        .into()],
    );

    check_parser(
        "while (true) {
            continue yield;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new("yield").into()]),
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
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new::<_, Box<str>>(None).into()]),
        )
        .into()],
    );
}

#[test]
fn new_line_block_empty_semicolon_insertion() {
    check_parser(
        "while (true) {
            continue
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new::<_, Box<str>>(None).into()]),
        )
        .into()],
    );
}
