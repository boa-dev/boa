use crate::{
    syntax::{
        ast::{
            node::{Block, Continue, WhileLoop},
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
        "while (true) continue;",
        vec![WhileLoop::new(Const::from(true), Continue::new::<_, Box<str>>(None)).into()],
        &mut interner,
    );
}

#[test]
fn new_line() {
    let mut interner = Interner::new();
    check_parser(
        "while (true)
            continue;",
        vec![WhileLoop::new(Const::from(true), Continue::new::<_, Box<str>>(None)).into()],
        &mut interner,
    );
}

#[test]
fn inline_block_semicolon_insertion() {
    let mut interner = Interner::new();
    check_parser(
        "while (true) {continue}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new::<_, Box<str>>(None).into()]),
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
            continue test
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new("test").into()]),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn inline_block() {
    let mut interner = Interner::new();
    check_parser(
        "while (true) {continue;}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new::<_, Box<str>>(None).into()]),
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
            continue test;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new("test").into()]),
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
            continue await;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new("await").into()]),
        )
        .into()],
        &mut interner,
    );

    let mut interner = Interner::new();
    check_parser(
        "while (true) {
            continue yield;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new("yield").into()]),
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
            continue;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new::<_, Box<str>>(None).into()]),
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
            continue
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Continue::new::<_, Box<str>>(None).into()]),
        )
        .into()],
        &mut interner,
    );
}
