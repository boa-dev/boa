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
    let mut interner = Interner::default();
    check_parser(
        "while (true) break;",
        vec![WhileLoop::new(Const::from(true), Node::Break(Break::new(None))).into()],
        &mut interner,
    );
}

#[test]
fn new_line() {
    let mut interner = Interner::default();
    check_parser(
        "while (true)
            break;",
        vec![WhileLoop::new(Const::from(true), Break::new(None)).into()],
        &mut interner,
    );
}

#[test]
fn inline_block_semicolon_insertion() {
    let mut interner = Interner::default();
    check_parser(
        "while (true) {break}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new(None).into()]),
        )
        .into()],
        &mut interner,
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
        &mut interner,
    );
}

#[test]
fn inline_block() {
    let mut interner = Interner::default();
    check_parser(
        "while (true) {break;}",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new(None).into()]),
        )
        .into()],
        &mut interner,
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
        &mut interner,
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
        &mut interner,
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
        &mut interner,
    );
}

#[test]
fn new_line_block_empty() {
    let mut interner = Interner::default();
    check_parser(
        "while (true) {
            break;
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new(None).into()]),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn new_line_block_empty_semicolon_insertion() {
    let mut interner = Interner::default();
    check_parser(
        "while (true) {
            break
        }",
        vec![WhileLoop::new(
            Const::from(true),
            Block::from(vec![Break::new(None).into()]),
        )
        .into()],
        &mut interner,
    );
}
