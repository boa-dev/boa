use crate::{
    string::utf16,
    syntax::{
        ast::{
            node::{
                field::GetConstField, BinOp, Block, Break, Call, Declaration, DeclarationList,
                DoWhileLoop, Identifier, UnaryOp, WhileLoop,
            },
            op::{self, AssignOp, CompOp},
            Const,
        },
        parser::tests::{check_invalid, check_parser},
    },
};
use boa_interner::Interner;

/// Checks do-while statement parsing.
#[test]
fn check_do_while() {
    let mut interner = Interner::default();
    check_parser(
        r#"do {
            a += 1;
        } while (true)"#,
        vec![DoWhileLoop::new(
            Block::from(vec![BinOp::new(
                AssignOp::Add,
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
                Const::from(1),
            )
            .into()]),
            Const::from(true),
        )
        .into()],
        interner,
    );
}

// Checks automatic semicolon insertion after do-while.
#[test]
fn check_do_while_semicolon_insertion() {
    let mut interner = Interner::default();
    check_parser(
        r#"var i = 0;
        do {console.log("hello");} while(i++ < 10) console.log("end");"#,
        vec![
            DeclarationList::Var(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("i", utf16!("i")),
                    Some(Const::from(0).into()),
                )]
                .into(),
            )
            .into(),
            DoWhileLoop::new(
                Block::from(vec![Call::new(
                    GetConstField::new(
                        Identifier::new(
                            interner.get_or_intern_static("console", utf16!("console")),
                        ),
                        interner.get_or_intern_static("log", utf16!("log")),
                    ),
                    vec![
                        Const::from(interner.get_or_intern_static("hello", utf16!("hello"))).into(),
                    ],
                )
                .into()]),
                BinOp::new(
                    CompOp::LessThan,
                    UnaryOp::new(
                        op::UnaryOp::IncrementPost,
                        Identifier::new(interner.get_or_intern_static("i", utf16!("i"))),
                    ),
                    Const::from(10),
                ),
            )
            .into(),
            Call::new(
                GetConstField::new(
                    Identifier::new(interner.get_or_intern_static("console", utf16!("console"))),
                    interner.get_or_intern_static("log", utf16!("log")),
                ),
                vec![Const::from(interner.get_or_intern_static("end", utf16!("end"))).into()],
            )
            .into(),
        ],
        interner,
    );
}

// Checks automatic semicolon insertion after do-while with no space between closing paren
// and next statement.
#[test]
fn check_do_while_semicolon_insertion_no_space() {
    let mut interner = Interner::default();
    check_parser(
        r#"var i = 0;
        do {console.log("hello");} while(i++ < 10)console.log("end");"#,
        vec![
            DeclarationList::Var(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("i", utf16!("i")),
                    Some(Const::from(0).into()),
                )]
                .into(),
            )
            .into(),
            DoWhileLoop::new(
                Block::from(vec![Call::new(
                    GetConstField::new(
                        Identifier::new(
                            interner.get_or_intern_static("console", utf16!("console")),
                        ),
                        interner.get_or_intern_static("log", utf16!("log")),
                    ),
                    vec![
                        Const::from(interner.get_or_intern_static("hello", utf16!("hello"))).into(),
                    ],
                )
                .into()]),
                BinOp::new(
                    CompOp::LessThan,
                    UnaryOp::new(
                        op::UnaryOp::IncrementPost,
                        Identifier::new(interner.get_or_intern_static("i", utf16!("i"))),
                    ),
                    Const::from(10),
                ),
            )
            .into(),
            Call::new(
                GetConstField::new(
                    Identifier::new(interner.get_or_intern_static("console", utf16!("console"))),
                    interner.get_or_intern_static("log", utf16!("log")),
                ),
                vec![Const::from(interner.get_or_intern_static("end", utf16!("end"))).into()],
            )
            .into(),
        ],
        interner,
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
        vec![WhileLoop::new(Const::from(true), Break::new(None)).into()],
        Interner::default(),
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
            Block::from(vec![Break::new(None).into()]),
            Const::Bool(true),
        )
        .into()],
        Interner::default(),
    );
}

/// Checks rejection of const bindings without init in for loops
#[test]
fn reject_const_no_init_for_loop() {
    check_invalid("for (const h;;);");
}
