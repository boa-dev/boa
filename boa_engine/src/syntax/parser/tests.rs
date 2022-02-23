//! Tests for the parser.

use super::Parser;
use crate::syntax::ast::{
    node::{
        field::GetConstField, object::PropertyDefinition, ArrowFunctionDecl, Assign, BinOp, Call,
        Declaration, DeclarationList, FormalParameter, FormalParameterList,
        FormalParameterListFlags, FunctionDecl, Identifier, If, New, Node, Object, Return,
        StatementList, UnaryOp,
    },
    op::{self, CompOp, LogOp, NumOp},
    Const,
};
use boa_interner::Interner;

/// Checks that the given JavaScript string gives the expected expression.
#[allow(clippy::unwrap_used)]
#[track_caller]
pub(super) fn check_parser<L>(js: &str, expr: L, interner: &mut Interner)
where
    L: Into<Box<[Node]>>,
{
    assert_eq!(
        Parser::new(js.as_bytes(), false)
            .parse_all(interner)
            .expect("failed to parse"),
        StatementList::from(expr)
    );
}

/// Checks that the given javascript string creates a parse error.
#[track_caller]
pub(super) fn check_invalid(js: &str) {
    let mut interner = Interner::default();
    assert!(Parser::new(js.as_bytes(), false)
        .parse_all(&mut interner)
        .is_err());
}

/// Should be parsed as `new Class().method()` instead of `new (Class().method())`
#[test]
fn check_construct_call_precedence() {
    let mut interner = Interner::default();
    check_parser(
        "new Date().getTime()",
        vec![Node::from(Call::new(
            GetConstField::new(
                New::from(Call::new(
                    Identifier::new(interner.get_or_intern_static("Date")),
                    vec![],
                )),
                interner.get_or_intern_static("getTime"),
            ),
            vec![],
        ))],
        &mut interner,
    );
}

#[test]
fn assign_operator_precedence() {
    let mut interner = Interner::default();
    check_parser(
        "a = a + 1",
        vec![Assign::new(
            Identifier::new(interner.get_or_intern_static("a")),
            BinOp::new(
                NumOp::Add,
                Identifier::new(interner.get_or_intern_static("a")),
                Const::from(1),
            ),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn hoisting() {
    let mut interner = Interner::default();
    let hello = interner.get_or_intern_static("hello");
    let a = interner.get_or_intern_static("a");
    check_parser(
        r"
            var a = hello();
            a++;

            function hello() { return 10 }",
        vec![
            FunctionDecl::new(
                hello,
                FormalParameterList::default(),
                vec![Return::new(Const::from(10), None).into()],
            )
            .into(),
            DeclarationList::Var(
                vec![Declaration::new_with_identifier(
                    a,
                    Node::from(Call::new(Identifier::new(hello), vec![])),
                )]
                .into(),
            )
            .into(),
            UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::new(a)).into(),
        ],
        &mut interner,
    );

    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a");
    check_parser(
        r"
            a = 10;
            a++;

            var a;",
        vec![
            Assign::new(Identifier::new(a), Const::from(10)).into(),
            UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::new(a)).into(),
            DeclarationList::Var(vec![Declaration::new_with_identifier(a, None)].into()).into(),
        ],
        &mut interner,
    );
}

#[test]
fn ambigous_regex_divide_expression() {
    let s = "1 / a === 1 / b";

    let mut interner = Interner::default();
    check_parser(
        s,
        vec![BinOp::new(
            CompOp::StrictEqual,
            BinOp::new(
                NumOp::Div,
                Const::Int(1),
                Identifier::new(interner.get_or_intern_static("a")),
            ),
            BinOp::new(
                NumOp::Div,
                Const::Int(1),
                Identifier::new(interner.get_or_intern_static("b")),
            ),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn two_divisions_in_expression() {
    let s = "a !== 0 || 1 / a === 1 / b;";

    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a");
    check_parser(
        s,
        vec![BinOp::new(
            LogOp::Or,
            BinOp::new(CompOp::StrictNotEqual, Identifier::new(a), Const::Int(0)),
            BinOp::new(
                CompOp::StrictEqual,
                BinOp::new(NumOp::Div, Const::Int(1), Identifier::new(a)),
                BinOp::new(
                    NumOp::Div,
                    Const::Int(1),
                    Identifier::new(interner.get_or_intern_static("b")),
                ),
            ),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn comment_semi_colon_insertion() {
    let s = r#"
    let a = 10 // Comment
    let b = 20;
    "#;

    let mut interner = Interner::default();
    check_parser(
        s,
        vec![
            DeclarationList::Let(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("a"),
                    Some(Const::Int(10).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Let(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("b"),
                    Some(Const::Int(20).into()),
                )]
                .into(),
            )
            .into(),
        ],
        &mut interner,
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

    let mut interner = Interner::default();
    check_parser(
        s,
        vec![
            DeclarationList::Let(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("a"),
                    Some(Const::Int(10).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Let(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("b"),
                    Some(Const::Int(20).into()),
                )]
                .into(),
            )
            .into(),
        ],
        &mut interner,
    );
}

#[test]
fn multiline_comment_no_lineterminator() {
    let s = r#"
    let a = 10; /* Test comment */ let b = 20;
    "#;

    let mut interner = Interner::default();
    check_parser(
        s,
        vec![
            DeclarationList::Let(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("a"),
                    Some(Const::Int(10).into()),
                )]
                .into(),
            )
            .into(),
            DeclarationList::Let(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("b"),
                    Some(Const::Int(20).into()),
                )]
                .into(),
            )
            .into(),
        ],
        &mut interner,
    );
}

#[test]
fn assignment_line_terminator() {
    let s = r#"
    let a = 3;

    a =
    5;
    "#;

    let mut interner = Interner::default();
    check_parser(
        s,
        vec![
            DeclarationList::Let(
                vec![Declaration::new_with_identifier(
                    interner.get_or_intern_static("a"),
                    Some(Const::Int(3).into()),
                )]
                .into(),
            )
            .into(),
            Assign::new(
                Identifier::new(interner.get_or_intern_static("a")),
                Const::from(5),
            )
            .into(),
        ],
        &mut interner,
    );
}

#[test]
fn assignment_multiline_terminator() {
    let s = r#"
    let a = 3;


    a =


    5;
    "#;

    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a");
    check_parser(
        s,
        vec![
            DeclarationList::Let(
                vec![Declaration::new_with_identifier(
                    a,
                    Some(Const::Int(3).into()),
                )]
                .into(),
            )
            .into(),
            Assign::new(Identifier::new(a), Const::from(5)).into(),
        ],
        &mut interner,
    );
}

#[test]
fn bracketed_expr() {
    let s = r#"(b)"#;

    let mut interner = Interner::default();
    check_parser(
        s,
        vec![Identifier::new(interner.get_or_intern_static("b")).into()],
        &mut interner,
    );
}

#[test]
fn increment_in_comma_op() {
    let s = r#"(b++, b)"#;

    let mut interner = Interner::default();
    let b = interner.get_or_intern_static("b");
    check_parser(
        s,
        vec![BinOp::new::<_, Node, Node>(
            op::BinOp::Comma,
            UnaryOp::new::<Node>(op::UnaryOp::IncrementPost, Identifier::new(b).into()).into(),
            Identifier::new(b).into(),
        )
        .into()],
        &mut interner,
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

    let mut interner = Interner::default();

    let object_properties = vec![
        PropertyDefinition::property(interner.get_or_intern_static("a"), Const::from(1)),
        PropertyDefinition::spread_object(Identifier::new(interner.get_or_intern_static("b"))),
    ];

    check_parser(
        s,
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("x"),
                Some(Object::from(object_properties).into()),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn spread_in_arrow_function() {
    let s = r#"
    (...b) => {
        b
    }
    "#;

    let mut interner = Interner::default();
    let b = interner.get_or_intern_static("b");
    check_parser(
        s,
        vec![ArrowFunctionDecl::new(
            None,
            FormalParameterList {
                parameters: Box::new([FormalParameter::new(
                    Declaration::new_with_identifier(b, None),
                    true,
                )]),
                flags: FormalParameterListFlags::empty()
                    .union(FormalParameterListFlags::HAS_REST_PARAMETER),
            },
            vec![Identifier::from(b).into()],
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn empty_statement() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a");
    check_parser(
        r"
            ;;var a = 10;
            if(a) ;
        ",
        vec![
            Node::Empty,
            DeclarationList::Var(
                vec![Declaration::new_with_identifier(
                    a,
                    Node::from(Const::from(10)),
                )]
                .into(),
            )
            .into(),
            Node::If(If::new::<_, _, Node, _>(
                Identifier::new(a),
                Node::Empty,
                None,
            )),
        ],
        &mut interner,
    );
}

#[test]
fn hashbang_use_strict_no_with() {
    let mut interner = Interner::default();
    check_parser(
        r#"#!\"use strict"
        "#,
        vec![],
        &mut interner,
    );
}

#[test]
#[ignore]
fn hashbang_use_strict_with_with_statement() {
    let mut interner = Interner::default();
    check_parser(
        r#"#!\"use strict"
        
        with({}) {}
        "#,
        vec![],
        &mut interner,
    );
}

#[test]
fn hashbang_comment() {
    let mut interner = Interner::default();
    check_parser(r"#!Comment Here", vec![], &mut interner);
}
