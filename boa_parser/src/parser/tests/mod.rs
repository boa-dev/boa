//! Tests for the parser.

mod format;

use std::convert::TryInto;

use crate::Parser;
use boa_ast::{
    declaration::{Declaration, LexicalDeclaration, VarDeclaration, Variable},
    expression::{
        access::SimplePropertyAccess,
        literal::{Literal, ObjectLiteral},
        operator::{
            assign::AssignOp,
            binary::{ArithmeticOp, BinaryOp, LogicalOp, RelationalOp},
            unary::UnaryOp,
            Assign, Binary, Unary,
        },
        Call, Identifier, New,
    },
    function::{
        ArrowFunction, FormalParameter, FormalParameterList, FormalParameterListFlags, Function,
    },
    property::PropertyDefinition,
    statement::{If, Return},
    Expression, Statement, StatementList, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks that the given JavaScript string gives the expected expression.
#[allow(clippy::unwrap_used)]
#[track_caller]
pub(super) fn check_parser<L>(js: &str, expr: L, interner: &mut Interner)
where
    L: Into<Box<[StatementListItem]>>,
{
    assert_eq!(
        Parser::new(js.as_bytes())
            .parse_all(interner)
            .expect("failed to parse"),
        StatementList::from(expr.into())
    );
}

/// Checks that the given javascript string creates a parse error.
#[track_caller]
pub(super) fn check_invalid(js: &str) {
    assert!(Parser::new(js.as_bytes())
        .parse_all(&mut Interner::default())
        .is_err());
}

/// Should be parsed as `new Class().method()` instead of `new (Class().method())`
#[test]
fn check_construct_call_precedence() {
    let interner = &mut Interner::default();
    check_parser(
        "new Date().getTime()",
        vec![Statement::Expression(Expression::from(Call::new(
            Expression::PropertyAccess(
                SimplePropertyAccess::new(
                    New::from(Call::new(
                        Identifier::new(interner.get_or_intern_static("Date", utf16!("Date")))
                            .into(),
                        Box::default(),
                    ))
                    .into(),
                    interner.get_or_intern_static("getTime", utf16!("getTime")),
                )
                .into(),
            ),
            Box::default(),
        )))
        .into()],
        interner,
    );
}

#[test]
fn assign_operator_precedence() {
    let interner = &mut Interner::default();
    check_parser(
        "a = a + 1",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Assign,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Binary::new(
                ArithmeticOp::Add.into(),
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                Literal::from(1).into(),
            )
            .into(),
        )))
        .into()],
        interner,
    );
}

#[test]
fn hoisting() {
    let interner = &mut Interner::default();
    let hello = interner
        .get_or_intern_static("hello", utf16!("hello"))
        .into();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_parser(
        r"
            var a = hello();
            a++;

            function hello() { return 10 }",
        vec![
            Declaration::Function(Function::new(
                Some(hello),
                FormalParameterList::default(),
                vec![Statement::Return(Return::new(Some(Literal::from(10).into()))).into()].into(),
            ))
            .into(),
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    a.into(),
                    Some(Call::new(hello.into(), Box::default()).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Expression(Expression::from(Unary::new(
                UnaryOp::IncrementPost,
                Identifier::new(a).into(),
            )))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_parser(
        r"
            a = 10;
            a++;

            var a;",
        vec![
            Statement::Expression(Expression::from(Assign::new(
                AssignOp::Assign,
                Identifier::new(a).into(),
                Literal::from(10).into(),
            )))
            .into(),
            Statement::Expression(Expression::from(Unary::new(
                UnaryOp::IncrementPost,
                Identifier::new(a).into(),
            )))
            .into(),
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(a.into(), None)]
                    .try_into()
                    .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn ambigous_regex_divide_expression() {
    let s = "1 / a === 1 / b";

    let interner = &mut Interner::default();
    check_parser(
        s,
        vec![Statement::Expression(Expression::from(Binary::new(
            RelationalOp::StrictEqual.into(),
            Binary::new(
                ArithmeticOp::Div.into(),
                Literal::Int(1).into(),
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            )
            .into(),
            Binary::new(
                ArithmeticOp::Div.into(),
                Literal::Int(1).into(),
                Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
            )
            .into(),
        )))
        .into()],
        interner,
    );
}

#[test]
fn two_divisions_in_expression() {
    let s = "a !== 0 || 1 / a === 1 / b;";

    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_parser(
        s,
        vec![Statement::Expression(Expression::from(Binary::new(
            LogicalOp::Or.into(),
            Binary::new(
                RelationalOp::StrictNotEqual.into(),
                Identifier::new(a).into(),
                Literal::Int(0).into(),
            )
            .into(),
            Binary::new(
                RelationalOp::StrictEqual.into(),
                Binary::new(
                    ArithmeticOp::Div.into(),
                    Literal::Int(1).into(),
                    Identifier::new(a).into(),
                )
                .into(),
                Binary::new(
                    ArithmeticOp::Div.into(),
                    Literal::Int(1).into(),
                    Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
                )
                .into(),
            )
            .into(),
        )))
        .into()],
        interner,
    );
}

#[test]
fn comment_semi_colon_insertion() {
    let s = r#"
    let a = 10 // Comment
    let b = 20;
    "#;

    let interner = &mut Interner::default();
    check_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::Int(10).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("b", utf16!("b")).into(),
                    Some(Literal::Int(20).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn multiline_comment_semi_colon_insertion() {
    let s = r#"
    let a = 10 /* Test
    Multiline
    Comment
    */ let b = 20;
    "#;

    let interner = &mut Interner::default();
    check_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::Int(10).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("b", utf16!("b")).into(),
                    Some(Literal::Int(20).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn multiline_comment_no_lineterminator() {
    let s = r#"
    let a = 10; /* Test comment */ let b = 20;
    "#;

    let interner = &mut Interner::default();
    check_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::Int(10).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("b", utf16!("b")).into(),
                    Some(Literal::Int(20).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn assignment_line_terminator() {
    let s = r#"
    let a = 3;

    a =
    5;
    "#;

    let interner = &mut Interner::default();
    check_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::Int(3).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Expression(Expression::from(Assign::new(
                AssignOp::Assign,
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                Literal::from(5).into(),
            )))
            .into(),
        ],
        interner,
    );
}

#[test]
fn assignment_multiline_terminator() {
    let s = r#"
    let a = 3;


    a =


    5;
    "#;

    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    a.into(),
                    Some(Literal::Int(3).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Expression(Expression::from(Assign::new(
                AssignOp::Assign,
                Identifier::new(a).into(),
                Literal::from(5).into(),
            )))
            .into(),
        ],
        interner,
    );
}

#[test]
fn bracketed_expr() {
    let s = r#"(b)"#;

    let interner = &mut Interner::default();
    check_parser(
        s,
        vec![Statement::Expression(Expression::from(Identifier::new(
            interner.get_or_intern_static("b", utf16!("b")),
        )))
        .into()],
        interner,
    );
}

#[test]
fn increment_in_comma_op() {
    let s = r#"(b++, b)"#;

    let interner = &mut Interner::default();
    let b = interner.get_or_intern_static("b", utf16!("b"));
    check_parser(
        s,
        vec![Statement::Expression(Expression::from(Binary::new(
            BinaryOp::Comma,
            Unary::new(UnaryOp::IncrementPost, Identifier::new(b).into()).into(),
            Identifier::new(b).into(),
        )))
        .into()],
        interner,
    );
}

#[test]
fn spread_in_object() {
    let s = r#"
    let x = {
      a: 1,
      ...b,
    }
    "#;

    let interner = &mut Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            interner.get_or_intern_static("a", utf16!("a")).into(),
            Literal::from(1).into(),
        ),
        PropertyDefinition::SpreadObject(
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        ),
    ];

    check_parser(
        s,
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("x", utf16!("x")).into(),
                Some(ObjectLiteral::from(object_properties).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn spread_in_arrow_function() {
    let s = r#"
    (...b) => {
        b
    }
    "#;

    let interner = &mut Interner::default();
    let b = interner.get_or_intern_static("b", utf16!("b"));
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(b.into(), None),
        true,
    ));
    assert_eq!(params.flags(), FormalParameterListFlags::HAS_REST_PARAMETER);
    assert_eq!(params.length(), 0);
    check_parser(
        s,
        vec![Statement::Expression(Expression::from(ArrowFunction::new(
            None,
            params,
            vec![Statement::Expression(Expression::from(Identifier::from(b))).into()].into(),
        )))
        .into()],
        interner,
    );
}

#[test]
fn empty_statement() {
    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_parser(
        r"
            ;;var a = 10;
            if(a) ;
        ",
        vec![
            Statement::Empty.into(),
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    a.into(),
                    Some(Literal::from(10).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::If(If::new(Identifier::new(a).into(), Statement::Empty, None)).into(),
        ],
        interner,
    );
}

#[test]
fn hashbang_use_strict_no_with() {
    check_parser(
        r#"#!\"use strict"
        "#,
        vec![],
        &mut Interner::default(),
    );
}

#[test]
#[ignore]
fn hashbang_use_strict_with_with_statement() {
    check_parser(
        r#"#!\"use strict"

        with({}) {}
        "#,
        vec![],
        &mut Interner::default(),
    );
}

#[test]
fn hashbang_comment() {
    check_parser(r"#!Comment Here", vec![], &mut Interner::default());
}
