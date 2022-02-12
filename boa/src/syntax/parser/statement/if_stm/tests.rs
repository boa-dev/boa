use crate::{
    syntax::{
        ast::{
            node::{Block, If, Node},
            Const,
        },
        parser::tests::check_parser,
    },
    Interner,
};

#[test]
fn if_without_else_block() {
    let mut interner = Interner::default();
    check_parser(
        "if (true) {}",
        vec![If::new::<_, _, Node, _>(Const::from(true), Block::from(Vec::new()), None).into()],
        &mut interner,
    );
}

#[test]
fn if_without_else_block_with_trailing_newline() {
    let mut interner = Interner::default();
    check_parser(
        "if (true) {}\n",
        vec![If::new::<_, _, Node, _>(Const::from(true), Block::from(Vec::new()), None).into()],
        &mut interner,
    );
}
