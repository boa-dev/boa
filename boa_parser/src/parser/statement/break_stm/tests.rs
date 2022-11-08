use crate::parser::tests::check_parser;
use boa_ast::{
    expression::literal::Literal,
    statement::{Block, Break, WhileLoop},
    Statement, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn inline() {
    check_parser(
        "while (true) break;",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Break::new(None).into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn new_line() {
    check_parser(
        "while (true)
            break;",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Break::new(None).into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn inline_block_semicolon_insertion() {
    check_parser(
        "while (true) {break}",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Block::from(vec![StatementListItem::Statement(Statement::Break(
                Break::new(None),
            ))])
            .into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn new_line_semicolon_insertion() {
    let interner = &mut Interner::default();
    check_parser(
        "while (true) {
            break test
        }",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Block::from(vec![StatementListItem::Statement(Statement::Break(
                Break::new(Some(interner.get_or_intern_static("test", utf16!("test")))),
            ))])
            .into(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn inline_block() {
    check_parser(
        "while (true) {break;}",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Block::from(vec![StatementListItem::Statement(Statement::Break(
                Break::new(None),
            ))])
            .into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn new_line_block() {
    let interner = &mut Interner::default();
    check_parser(
        "while (true) {
            break test;
        }",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Block::from(vec![StatementListItem::Statement(Statement::Break(
                Break::new(Some(interner.get_or_intern_static("test", utf16!("test")))),
            ))])
            .into(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn reserved_label() {
    let interner = &mut Interner::default();
    check_parser(
        "while (true) {
            break await;
        }",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Block::from(vec![StatementListItem::Statement(Statement::Break(
                Break::new(Some(
                    interner.get_or_intern_static("await", utf16!("await")),
                )),
            ))])
            .into(),
        ))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "while (true) {
            break yield;
        }",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Block::from(vec![StatementListItem::Statement(Statement::Break(
                Break::new(Some(
                    interner.get_or_intern_static("yield", utf16!("yield")),
                )),
            ))])
            .into(),
        ))
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
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Block::from(vec![StatementListItem::Statement(Statement::Break(
                Break::new(None),
            ))])
            .into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn new_line_block_empty_semicolon_insertion() {
    check_parser(
        "while (true) {
            break
        }",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Block::from(vec![StatementListItem::Statement(Statement::Break(
                Break::new(None),
            ))])
            .into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}
