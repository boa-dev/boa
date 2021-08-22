use crate::syntax::{
    ast::{
        node::{
            field::GetConstField, BinOp, Block, Break, Call, Declaration, DeclarationList,
            DoWhileLoop, Identifier, UnaryOp, WhileLoop,
        },
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
        vec![DoWhileLoop::new(
            Block::from(vec![BinOp::new(
                AssignOp::Add,
                Identifier::from("a"),
                Const::from(1),
            )
            .into()]),
            Const::from(true),
        )
        .into()],
    );
}

// Checks automatic semicolon insertion after do-while.
#[test]
fn check_do_while_semicolon_insertion() {
    check_parser(
        r#"var i = 0;
        do {console.log("hello");} while(i++ < 10) console.log("end");"#,
        vec![
            DeclarationList::Var(
                vec![Declaration::new_with_identifier(
                    "i",
                    Some(Const::from(0).into()),
                )]
                .into(),
            )
            .into(),
            DoWhileLoop::new(
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
            )
            .into(),
            Call::new(
                GetConstField::new(Identifier::from("console"), "log"),
                vec![Const::from("end").into()],
            )
            .into(),
        ],
    );
}

// Checks automatic semicolon insertion after do-while with no space between closing paren
// and next statement.
#[test]
fn check_do_while_semicolon_insertion_no_space() {
    check_parser(
        r#"var i = 0;
        do {console.log("hello");} while(i++ < 10)console.log("end");"#,
        vec![
            DeclarationList::Var(
                vec![Declaration::new_with_identifier(
                    "i",
                    Some(Const::from(0).into()),
                )]
                .into(),
            )
            .into(),
            DoWhileLoop::new(
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
            )
            .into(),
            Call::new(
                GetConstField::new(Identifier::from("console"), "log"),
                vec![Const::from("end").into()],
            )
            .into(),
        ],
    );
}

/// Checks parsing of a while statement which is seperated out with line terminators.
#[test]
fn while_spaces() {
    check_parser(
        r#"

        while

        (

        true

        )

        break;

        "#,
        vec![WhileLoop::new(Const::from(true), Break::new::<_, Box<str>>(None)).into()],
    );
}

/// Checks parsing of a while statement which is seperated out with line terminators.
#[test]
fn do_while_spaces() {
    check_parser(
        r#"

        do

        {

            break;

        }

        while (true)

        "#,
        vec![DoWhileLoop::new(
            Block::from(vec![Break::new::<Option<Box<str>>, Box<str>>(None).into()]),
            Const::Bool(true),
        )
        .into()],
    );
}
