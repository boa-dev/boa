use crate::{
    syntax::{
        ast::{
            node::{Block, Break, Node, WhileLoop},
            Const,
        },
        parser::tests::check_parser,
    },
    Interner,
};

#[test]
fn inline() {
    let mut interner = Interner::new();
    check_parser(
        "while (true) break;",
        vec![WhileLoop::new(
            Const::from(true),
            Node::Break(Break::new::<_, Box<str>>(None)),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn new_line() {
    let mut interner = Interner::new();
    check_parser(
        "while (true)
            break;",
        vec![WhileLoop::new(Const::from(true), Break::new::<_, Box<str>>(None)).into()],
        &mut interner,
    );
}

#[test]
fn inline_block_semicolon_insertion() {
    let mut interner = Interner::new();
    check_parser(
        "while (true) {break}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new::<_, Box<str>>(None).into()]),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn new_line_semicolon_insertion() {
    let mut interner = Interner::new();
    check_parser(
        "while (true) {
            break test
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new("test").into()]),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn inline_block() {
    let mut interner = Interner::new();
    check_parser(
        "while (true) {break;}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new::<_, Box<str>>(None).into()]),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn new_line_block() {
    let mut interner = Interner::new();
    check_parser(
        "while (true) {
            break test;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new("test").into()]),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn reserved_label() {
    let mut interner = Interner::new();
    check_parser(
        "while (true) {
            break await;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new("await").into()]),
        )
        .into()],
        &mut interner,
    );

    let mut interner = Interner::new();
    check_parser(
        "while (true) {
            break yield;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new("yield").into()]),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn new_line_block_empty() {
    let mut interner = Interner::new();
    check_parser(
        "while (true) {
            break;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new::<_, Box<str>>(None).into()]),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn new_line_block_empty_semicolon_insertion() {
    let mut interner = Interner::new();
    check_parser(
        "while (true) {
            break
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new::<_, Box<str>>(None).into()]),
        )
        .into()],
        &mut interner,
    );
}
