use crate::syntax::{
    ast::{
        node::{Block, Break, Node, WhileLoop},
        Const,
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;

#[test]
fn inline() {
    check_parser(
        "while (true) break;",
        vec![WhileLoop::new(Const::from(true), Node::Break(Break::new(None))).into()],
        Interner::default(),
    );
}

#[test]
fn new_line() {
    check_parser(
        "while (true)
            break;",
        vec![WhileLoop::new(Const::from(true), Break::new(None)).into()],
        Interner::default(),
    );
}

#[test]
fn inline_block_semicolon_insertion() {
    check_parser(
        "while (true) {break}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new(None).into()]),
        )
        .into()],
        Interner::default(),
    );
}

#[test]
fn new_line_semicolon_insertion() {
    let mut interner = Interner::default();
    check_parser(
        "while (true) {
            break test
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![
                Break::new(interner.get_or_intern_static("test")).into()
            ]),
        )
        .into()],
        interner,
    );
}

#[test]
fn inline_block() {
    check_parser(
        "while (true) {break;}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new(None).into()]),
        )
        .into()],
        Interner::default(),
    );
}

#[test]
fn new_line_block() {
    let mut interner = Interner::default();
    check_parser(
        "while (true) {
            break test;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![
                Break::new(interner.get_or_intern_static("test")).into()
            ]),
        )
        .into()],
        interner,
    );
}

#[test]
fn reserved_label() {
    let mut interner = Interner::default();
    check_parser(
        "while (true) {
            break await;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![
                Break::new(interner.get_or_intern_static("await")).into()
            ]),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "while (true) {
            break yield;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![
                Break::new(interner.get_or_intern_static("yield")).into()
            ]),
        )
        .into()],
        interner,
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
            Block::from(vec![Break::new(None).into()]),
        )
        .into()],
        Interner::default(),
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
            Block::from(vec![Break::new(None).into()]),
        )
        .into()],
        Interner::default(),
    );
}
