use crate::parser::tests::check_script_parser;
use boa_ast::{
    expression::literal::Literal,
    statement::{Block, Continue, Labelled, LabelledItem, WhileLoop},
    Statement, StatementListItem,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;

const PSEUDO_LINEAR_POS: boa_ast::LinearPosition = boa_ast::LinearPosition::new(0);

fn stmt_block_continue_only(continue_stmt: Continue) -> Statement {
    Block::from((
        vec![StatementListItem::Statement(Statement::Continue(
            continue_stmt,
        ))],
        PSEUDO_LINEAR_POS,
    ))
    .into()
}

#[test]
fn inline() {
    check_script_parser(
        "while (true) continue;",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Continue::new(None).into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn new_line() {
    check_script_parser(
        "while (true)
            continue;",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Continue::new(None).into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn inline_block_semicolon_insertion() {
    check_script_parser(
        "while (true) {continue}",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            stmt_block_continue_only(Continue::new(None)),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn new_line_semicolon_insertion() {
    let interner = &mut Interner::default();
    check_script_parser(
        "test: while (true) {
            continue test
        }",
        vec![Statement::Labelled(Labelled::new(
            LabelledItem::Statement(Statement::WhileLoop(WhileLoop::new(
                Literal::from(true).into(),
                stmt_block_continue_only(Continue::new(Some(
                    interner.get_or_intern_static("test", utf16!("test")),
                ))),
            ))),
            interner.get_or_intern_static("test", utf16!("test")),
        ))
        .into()],
        interner,
    );
}

#[test]
fn inline_block() {
    check_script_parser(
        "while (true) {continue;}",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            stmt_block_continue_only(Continue::new(None)),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn new_line_block() {
    let interner = &mut Interner::default();
    check_script_parser(
        "test: while (true) {
            continue test;
        }",
        vec![Statement::Labelled(Labelled::new(
            LabelledItem::Statement(Statement::WhileLoop(WhileLoop::new(
                Literal::from(true).into(),
                stmt_block_continue_only(Continue::new(Some(
                    interner.get_or_intern_static("test", utf16!("test")),
                ))),
            ))),
            interner.get_or_intern_static("test", utf16!("test")),
        ))
        .into()],
        interner,
    );
}

#[test]
fn reserved_label() {
    let interner = &mut Interner::default();
    check_script_parser(
        "await: while (true) {
            continue await;
        }",
        vec![Statement::Labelled(Labelled::new(
            LabelledItem::Statement(Statement::WhileLoop(WhileLoop::new(
                Literal::from(true).into(),
                stmt_block_continue_only(Continue::new(Some(Sym::AWAIT))),
            ))),
            Sym::AWAIT,
        ))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "yield: while (true) {
            continue yield;
        }",
        vec![Statement::Labelled(Labelled::new(
            LabelledItem::Statement(Statement::WhileLoop(WhileLoop::new(
                Literal::from(true).into(),
                stmt_block_continue_only(Continue::new(Some(Sym::YIELD))),
            ))),
            Sym::YIELD,
        ))
        .into()],
        interner,
    );
}

#[test]
fn new_line_block_empty() {
    check_script_parser(
        "while (true) {
            continue;
        }",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            stmt_block_continue_only(Continue::new(None)),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn new_line_block_empty_semicolon_insertion() {
    check_script_parser(
        "while (true) {
            continue
        }",
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            stmt_block_continue_only(Continue::new(None)),
        ))
        .into()],
        &mut Interner::default(),
    );
}
