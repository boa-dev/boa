use crate::syntax::{
    ast::{
        expression::{
            access::PropertyAccess,
            literal::Literal,
            operator::{
                assign::op::AssignOp, binary::op::RelationalOp, unary::op::UnaryOp, Assign, Binary,
                Unary,
            },
            Call, Identifier,
        },
        statement::{
            declaration::{Declaration, DeclarationList},
            Block, Break, DoWhileLoop, WhileLoop,
        },
        Expression,
    },
    parser::tests::{check_invalid, check_parser},
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks do-while statement parsing.
#[test]
fn check_do_while() {
    let mut interner = Interner::default();
    check_parser(
        r#"do {
            a += 1;
        } while (true)"#,
        vec![DoWhileLoop::new(
            Block::from(vec![Expression::from(Assign::new(
                AssignOp::Add,
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                Literal::from(1).into(),
            ))
            .into()])
            .into(),
            Literal::from(true).into(),
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
                vec![Declaration::from_identifier(
                    interner.get_or_intern_static("i", utf16!("i")).into(),
                    Some(Literal::from(0).into()),
                )]
                .into(),
            )
            .into(),
            DoWhileLoop::new(
                Block::from(vec![Expression::from(Call::new(
                    PropertyAccess::new(
                        Identifier::new(
                            interner.get_or_intern_static("console", utf16!("console")),
                        )
                        .into(),
                        interner.get_or_intern_static("log", utf16!("log")),
                    )
                    .into(),
                    vec![
                        Literal::from(interner.get_or_intern_static("hello", utf16!("hello")))
                            .into(),
                    ]
                    .into(),
                ))
                .into()])
                .into(),
                Binary::new(
                    RelationalOp::LessThan.into(),
                    Unary::new(
                        UnaryOp::IncrementPost,
                        Identifier::new(interner.get_or_intern_static("i", utf16!("i"))).into(),
                    )
                    .into(),
                    Literal::from(10).into(),
                )
                .into(),
            )
            .into(),
            Expression::from(Call::new(
                PropertyAccess::new(
                    Identifier::new(interner.get_or_intern_static("console", utf16!("console")))
                        .into(),
                    interner.get_or_intern_static("log", utf16!("log")),
                )
                .into(),
                vec![Literal::from(interner.get_or_intern_static("end", utf16!("end"))).into()]
                    .into(),
            ))
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
                vec![Declaration::from_identifier(
                    interner.get_or_intern_static("i", utf16!("i")).into(),
                    Some(Literal::from(0).into()),
                )]
                .into(),
            )
            .into(),
            DoWhileLoop::new(
                Block::from(vec![Expression::from(Call::new(
                    PropertyAccess::new(
                        Identifier::new(
                            interner.get_or_intern_static("console", utf16!("console")),
                        )
                        .into(),
                        interner.get_or_intern_static("log", utf16!("log")),
                    )
                    .into(),
                    vec![
                        Literal::from(interner.get_or_intern_static("hello", utf16!("hello")))
                            .into(),
                    ]
                    .into(),
                ))
                .into()])
                .into(),
                Binary::new(
                    RelationalOp::LessThan.into(),
                    Unary::new(
                        UnaryOp::IncrementPost,
                        Identifier::new(interner.get_or_intern_static("i", utf16!("i"))).into(),
                    )
                    .into(),
                    Literal::from(10).into(),
                )
                .into(),
            )
            .into(),
            Expression::from(Call::new(
                PropertyAccess::new(
                    Identifier::new(interner.get_or_intern_static("console", utf16!("console")))
                        .into(),
                    interner.get_or_intern_static("log", utf16!("log")),
                )
                .into(),
                vec![Literal::from(interner.get_or_intern_static("end", utf16!("end"))).into()]
                    .into(),
            ))
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
        vec![WhileLoop::new(Literal::from(true).into(), Break::new(None).into()).into()],
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
            Block::from(vec![Break::new(None).into()]).into(),
            Literal::Bool(true).into(),
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
