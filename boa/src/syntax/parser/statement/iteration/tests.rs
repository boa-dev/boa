use crate::syntax::{
    ast::{
        node::{BinOp, Block, Call, Identifier, Node, UnaryOp, VarDecl, VarDeclList, field::GetConstField},
        op::{self, AssignOp, CompOp},
        Const,
    },
    parser::tests::check_parser,
};

/// Checks do-while statement parsing.
#[test]
fn check_do_while() {
    check_parser(
        r#"do {
            a += 1;
        } while (true)"#,
        vec![Node::do_while_loop(
            Block::from(vec![BinOp::new(
                AssignOp::Add,
                Identifier::from("a"),
                Const::from(1),
            )
            .into()]),
            Const::from(true),
        )],
    );
}

// Checks automatic semicolon insertion after do-while.
#[test]
fn check_do_while_semicolon_insertion() {
    check_parser(
        r#"var i = 0;
        do {console.log("hello");} while(i++ < 10) console.log("end");"#,
        vec![
            VarDeclList::from(vec![VarDecl::new("i", Some(Const::from(0).into()))]).into(),
            Node::do_while_loop(
                Block::from(vec![Call::new(
                    GetConstField::new(Identifier::from("console"), "log"),
                    vec![Const::from("hello").into()],
                )
                .into()]),
                BinOp::new(
                    CompOp::LessThan,
                    UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::from("i")),
                    Const::from(10),
                ),
            ),
            Call::new(
                GetConstField::new(Identifier::from("console"), "log"),
                vec![Const::from("end").into()],
            )
            .into(),
        ],
    );
}
