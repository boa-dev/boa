use crate::syntax::{
    ast::{
        expression::literal::Literal,
        statement::{Block, If},
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;

#[test]
fn if_without_else_block() {
    check_parser(
        "if (true) {}",
        vec![If::new(
            Literal::from(true).into(),
            Block::from(Vec::new()).into(),
            None,
        )
        .into()],
        Interner::default(),
    );
}

#[test]
fn if_without_else_block_with_trailing_newline() {
    check_parser(
        "if (true) {}\n",
        vec![If::new(
            Literal::from(true).into(),
            Block::from(Vec::new()).into(),
            None,
        )
        .into()],
        Interner::default(),
    );
}
