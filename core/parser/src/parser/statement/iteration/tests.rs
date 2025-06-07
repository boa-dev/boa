use crate::parser::tests::{check_invalid_script, check_script_parser};
use boa_ast::{
    declaration::{VarDeclaration, Variable},
    expression::{
        access::SimplePropertyAccess,
        literal::Literal,
        operator::{
            assign::AssignOp,
            binary::RelationalOp,
            update::{UpdateOp, UpdateTarget},
            Assign, Binary, Update,
        },
        Call, Identifier,
    },
    statement::{Block, Break, DoWhileLoop, WhileLoop},
    Expression, Span, Statement, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;
use indoc::indoc;

const PSEUDO_LINEAR_POS: boa_ast::LinearPosition = boa_ast::LinearPosition::new(0);

/// Checks do-while statement parsing.
#[test]
fn check_do_while() {
    let interner = &mut Interner::default();
    check_script_parser(
        indoc! {"
            do {
                a += 1;
            } while (true)
        "},
        vec![Statement::DoWhileLoop(DoWhileLoop::new(
            Statement::Block(
                (
                    vec![StatementListItem::Statement(Statement::Expression(
                        Expression::from(Assign::new(
                            AssignOp::Add,
                            Identifier::new(
                                interner.get_or_intern_static("a", utf16!("a")),
                                Span::new((2, 5), (2, 6)),
                            )
                            .into(),
                            Literal::new(1, Span::new((2, 10), (2, 11))).into(),
                        )),
                    ))],
                    PSEUDO_LINEAR_POS,
                )
                    .into(),
            ),
            Literal::new(true, Span::new((3, 10), (3, 14))).into(),
        ))
        .into()],
        interner,
    );
}

// Checks automatic semicolon insertion after do-while.
#[test]
fn check_do_while_semicolon_insertion() {
    let interner = &mut Interner::default();
    check_script_parser(
        indoc! {r#"
            var i = 0;
            do {console.log("hello");} while(i++ < 10) console.log("end");
        "#},
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("i", utf16!("i")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(0, Span::new((1, 9), (1, 10))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::DoWhileLoop(DoWhileLoop::new(
                Statement::Block(
                    (
                        vec![StatementListItem::Statement(Statement::Expression(
                            Expression::from(Call::new(
                                Expression::PropertyAccess(
                                    SimplePropertyAccess::new(
                                        Identifier::new(
                                            interner
                                                .get_or_intern_static("console", utf16!("console")),
                                            Span::new((2, 5), (2, 12)),
                                        )
                                        .into(),
                                        interner.get_or_intern_static("log", utf16!("log")),
                                    )
                                    .into(),
                                ),
                                vec![Literal::new(
                                    interner.get_or_intern_static("hello", utf16!("hello")),
                                    Span::new((2, 17), (2, 24)),
                                )
                                .into()]
                                .into(),
                            )),
                        ))],
                        PSEUDO_LINEAR_POS,
                    )
                        .into(),
                ),
                Binary::new(
                    RelationalOp::LessThan.into(),
                    Update::new(
                        UpdateOp::IncrementPost,
                        UpdateTarget::Identifier(Identifier::new(
                            interner.get_or_intern_static("i", utf16!("i")),
                            Span::new((2, 34), (2, 35)),
                        )),
                    )
                    .into(),
                    Literal::new(10, Span::new((2, 40), (2, 42))).into(),
                )
                .into(),
            ))
            .into(),
            Statement::Expression(Expression::from(Call::new(
                Expression::PropertyAccess(
                    SimplePropertyAccess::new(
                        Identifier::new(
                            interner.get_or_intern_static("console", utf16!("console")),
                            Span::new((2, 44), (2, 51)),
                        )
                        .into(),
                        interner.get_or_intern_static("log", utf16!("log")),
                    )
                    .into(),
                ),
                vec![Literal::new(
                    interner.get_or_intern_static("end", utf16!("end")),
                    Span::new((2, 56), (2, 61)),
                )
                .into()]
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
    check_script_parser(
        indoc! {r#"
            var i = 0;
            do {console.log("hello");} while(i++ < 10)console.log("end");
        "#},
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("i", utf16!("i")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(0, Span::new((1, 9), (1, 10))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::DoWhileLoop(DoWhileLoop::new(
                Statement::Block(
                    (
                        vec![StatementListItem::Statement(Statement::Expression(
                            Expression::from(Call::new(
                                Expression::PropertyAccess(
                                    SimplePropertyAccess::new(
                                        Identifier::new(
                                            interner
                                                .get_or_intern_static("console", utf16!("console")),
                                            Span::new((2, 5), (2, 12)),
                                        )
                                        .into(),
                                        interner.get_or_intern_static("log", utf16!("log")),
                                    )
                                    .into(),
                                ),
                                vec![Literal::new(
                                    interner.get_or_intern_static("hello", utf16!("hello")),
                                    Span::new((2, 17), (2, 24)),
                                )
                                .into()]
                                .into(),
                            )),
                        ))],
                        PSEUDO_LINEAR_POS,
                    )
                        .into(),
                ),
                Binary::new(
                    RelationalOp::LessThan.into(),
                    Update::new(
                        UpdateOp::IncrementPost,
                        UpdateTarget::Identifier(Identifier::new(
                            interner.get_or_intern_static("i", utf16!("i")),
                            Span::new((2, 34), (2, 35)),
                        )),
                    )
                    .into(),
                    Literal::new(10, Span::new((2, 40), (2, 42))).into(),
                )
                .into(),
            ))
            .into(),
            Statement::Expression(Expression::from(Call::new(
                Expression::PropertyAccess(
                    SimplePropertyAccess::new(
                        Identifier::new(
                            interner.get_or_intern_static("console", utf16!("console")),
                            Span::new((2, 43), (2, 50)),
                        )
                        .into(),
                        interner.get_or_intern_static("log", utf16!("log")),
                    )
                    .into(),
                ),
                vec![Literal::new(
                    interner.get_or_intern_static("end", utf16!("end")),
                    Span::new((2, 55), (2, 60)),
                )
                .into()]
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
    check_script_parser(
        indoc! {"

            while

            (

            true

            )

            break;

        "},
        vec![Statement::WhileLoop(WhileLoop::new(
            Literal::new(true, Span::new((6, 1), (6, 5))).into(),
            Break::new(None).into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}

/// Checks parsing of a while statement which is seperated out with line terminators.
#[test]
fn do_while_spaces() {
    check_script_parser(
        indoc! {"

        do

        {

            break;

        }

        while (true)

        "},
        vec![Statement::DoWhileLoop(DoWhileLoop::new(
            Block::from((
                vec![StatementListItem::Statement(Statement::Break(Break::new(
                    None,
                )))],
                PSEUDO_LINEAR_POS,
            ))
            .into(),
            Literal::new(true, Span::new((10, 8), (10, 12))).into(),
        ))
        .into()],
        &mut Interner::default(),
    );
}

/// Checks rejection of const bindings without init in for loops
#[test]
fn reject_const_no_init_for_loop() {
    check_invalid_script("for (const h;;);");
}

/// Checks rejection of for await .. in loops
#[test]
fn reject_for_await_in_loop() {
    check_invalid_script("for await (x in [1,2,3]);");
}
