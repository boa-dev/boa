use crate::syntax::{
    ast::{
        node::{Block, If, Node},
        Const,
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;

#[test]
fn if_without_else_block() {
    check_parser(
        "if (true) {}",
        vec![If::new::<_, _, Node, _>(Const::from(true), Block::from(Vec::new()), None).into()],
        Interner::default(),
    );
}

#[test]
fn if_without_else_block_with_trailing_newline() {
    check_parser(
        "if (true) {}\n",
        vec![If::new::<_, _, Node, _>(Const::from(true), Block::from(Vec::new()), None).into()],
        Interner::default(),
    );
}
