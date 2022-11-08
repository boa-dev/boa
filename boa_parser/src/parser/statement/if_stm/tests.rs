use crate::parser::tests::check_parser;
use boa_ast::{
    expression::literal::Literal,
    statement::{Block, If},
    Statement,
};
use boa_interner::Interner;

#[test]
fn if_without_else_block() {
    check_parser(
        "if (true) {}",
        vec![Statement::If(If::new(
            Literal::from(true).into(),
            Block::from(Vec::new()).into(),
            None,
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn if_without_else_block_with_trailing_newline() {
    check_parser(
        "if (true) {}\n",
        vec![Statement::If(If::new(
            Literal::from(true).into(),
            Block::from(Vec::new()).into(),
            None,
        ))
        .into()],
        &mut Interner::default(),
    );
}
