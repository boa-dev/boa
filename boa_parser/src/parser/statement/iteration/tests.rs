use crate::parser::tests::{check_invalid, check_parser};
use boa_ast::{
    declaration::{VarDeclaration, Variable},
    expression::{
        access::SimplePropertyAccess,
        literal::Literal,
        operator::{assign::AssignOp, binary::RelationalOp, unary::UnaryOp, Assign, Binary, Unary},
        Call, Identifier,
    },
    statement::{Block, Break, DoWhileLoop, WhileLoop},
    Expression, Statement, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks do-while statement parsing.
#[test]
fn check_do_while() {
    let interner = &mut Interner::default();
    check_parser(
        r#"do {
            a += 1;
        } while (true)"#,
        vec![Statement::DoWhileLoop(DoWhileLoop::new(
            Statement::Block(
                vec![StatementListItem::Statement(Statement::Expression(
                    Expression::from(Assign::new(
                        AssignOp::Add,
                        Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                        Literal::from(1).into(),
                    )),
                ))]
                .into(),
            ),
            Literal::from(true).into(),
        ))
        .into()],
        interner,
    );
}

// Checks automatic semicolon insertion after do-while.
#[test]
fn check_do_while_semicolon_insertion() {
    let interner = &mut Interner::default();
    check_parser(
        r#"var i = 0;
        do {console.log("hello");} while(i++ < 10) console.log("end");"#,
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("i", utf16!("i")).into(),
                    Some(Literal::from(0).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::DoWhileLoop(DoWhileLoop::new(
                Statement::Block(
                    vec![StatementListItem::Statement(Statement::Expression(
                        Expression::from(Call::new(
                            Expression::PropertyAccess(
                                SimplePropertyAccess::new(
                                    Identifier::new(
                                        interner.get_or_intern_static("console", utf16!("console")),
                                    )
                                    .into(),
                                    interner.get_or_intern_static("log", utf16!("log")),
                                )
                                .into(),
                            ),
                            vec![Literal::from(
                                interner.get_or_intern_static("hello", utf16!("hello")),
                            )
                            .into()]
                            .into(),
                        )),
                    ))]
                    .into(),
                ),
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
            ))
            .into(),
            Statement::Expression(Expression::from(Call::new(
                Expression::PropertyAccess(
                    SimplePropertyAccess::new(
                        Identifier::new(
                            interner.get_or_intern_static("console", utf16!("console")),
                        )
                        .into(),
                        interner.get_or_intern_static("log", utf16!("log")),
                    )
                    .into(),
                ),
                vec![Literal::from(interner.get_or_intern_static("end", utf16!("end"))).into()]
                    .into(),
            )))
            .into(),
        ],
        interner,
    );
}

// Checks automatic semicolon insertion after do-while with no space between closing paren
// and next statement.
#[test]
fn check_do_while_semicolon_insertion_no_space() {
    let interner = &mut Interner::default();
    check_parser(
        r#"var i = 0;
        do {console.log("hello");} while(i++ < 10)console.log("end");"#,
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("i", utf16!("i")).into(),
                    Some(Literal::from(0).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::DoWhileLoop(DoWhileLoop::new(
                Statement::Block(
                    vec![StatementListItem::Statement(Statement::Expression(
                        Expression::from(Call::new(
                            Expression::PropertyAccess(
                                SimplePropertyAccess::new(
                                    Identifier::new(
                                        interner.get_or_intern_static("console", utf16!("console")),
                                    )
                                    .into(),
                                    interner.get_or_intern_static("log", utf16!("log")),
                                )
                                .into(),
                            ),
                            vec![Literal::from(
                                interner.get_or_intern_static("hello", utf16!("hello")),
                            )
                            .into()]
                            .into(),
                        )),
                    ))]
                    .into(),
                ),
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
            ))
            .into(),
            Statement::Expression(Expression::from(Call::new(
                Expression::PropertyAccess(
                    SimplePropertyAccess::new(
                        Identifier::new(
                            interner.get_or_intern_static("console", utf16!("console")),
                        )
                        .into(),
                        interner.get_or_intern_static("log", utf16!("log")),
                    )
                    .into(),
                ),
                vec![Literal::from(interner.get_or_intern_static("end", utf16!("end"))).into()]
                    .into(),
            )))
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
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::from(true).into(),
            Break::new(None).into(),
        ))
        .into()],
        &mut Interner::default(),
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
        vec![Statement::DoWhileLoop(DoWhileLoop::new(
            Block::from(vec![StatementListItem::Statement(Statement::Break(
                Break::new(None),
            ))])
            .into(),
            Literal::Bool(true).into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}

/// Checks rejection of const bindings without init in for loops
#[test]
fn reject_const_no_init_for_loop() {
    check_invalid("for (const h;;);");
}

/// Checks rejection of for await .. in loops
#[test]
fn reject_for_await_in_loop() {
    check_invalid("for await (x in [1,2,3]);");
}
