use crate::syntax::{
    ast::{
        node::{Block, Node},
        Const,
    },
    parser::tests::check_parser,
};

#[test]
fn if_without_else_block() {
    check_parser(
        "if (true) {}",
        vec![Node::if_node::<_, _, Node, _>(
            Const::from(true),
            Block::from(Vec::new()),
            None,
        )],
    );
}

#[test]
fn if_without_else_block_with_trailing_newline() {
    check_parser(
        "if (true) {}\n",
        vec![Node::if_node::<_, _, Node, _>(
            Const::from(true),
            Block::from(Vec::new()),
            None,
        )],
    );
}
