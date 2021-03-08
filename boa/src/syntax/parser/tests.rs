//! Tests for the parser.

use super::Parser;
use crate::syntax::ast::{
    node::{
        field::GetConstField, ArrowFunctionDecl, Assign, BinOp, Call, FormalParameter,
        FunctionDecl, Identifier, LetDecl, LetDeclList, New, Node, Return, StatementList, UnaryOp,
        VarDecl, VarDeclList, If
    },
    op::{self, CompOp, LogOp, NumOp},
    Const,
};

/// Checks that the given JavaScript string gives the expected expression.
#[allow(clippy::unwrap_used)]
#[track_caller]
pub(super) fn check_parser<L>(js: &str, expr: L)
where
    L: Into<Box<[Node]>>,
{
    assert_eq!(
        Parser::new(js.as_bytes(), false)
            .parse_all()
            .expect("failed to parse"),
        StatementList::from(expr)
    );
}

/// Checks that the given javascript string creates a parse error.
#[track_caller]
pub(super) fn check_invalid(js: &str) {
    assert!(Parser::new(js.as_bytes(), false).parse_all().is_err());
}

/// Should be parsed as `new Class().method()` instead of `new (Class().method())`
#[test]
fn check_construct_call_precedence() {
    check_parser(
        "new Date().getTime()",
        vec![Node::from(Call::new(
            GetConstField::new(
                New::from(Call::new(Identifier::from("Date"), vec![])),
                "getTime",
            ),
            vec![],
        ))],
    );
}

#[test]
fn assign_operator_precedence() {
    check_parser(
        "a = a + 1",
        vec![Assign::new(
            Identifier::from("a"),
            BinOp::new(NumOp::Add, Identifier::from("a"), Const::from(1)),
        )
        .into()],
    );
}

#[test]
fn hoisting() {
    check_parser(
        r"
            var a = hello();
            a++;

            function hello() { return 10 }",
        vec![
            FunctionDecl::new(
                Box::from("hello"),
                vec![],
                vec![Return::new(Const::from(10), None).into()],
            )
            .into(),
            VarDeclList::from(vec![VarDecl::new(
                "a",
                Node::from(Call::new(Identifier::from("hello"), vec![])),
            )])
            .into(),
            UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::from("a")).into(),
        ],
    );

    check_parser(
        r"
            a = 10;
            a++;

            var a;",
        vec![
            Assign::new(Identifier::from("a"), Const::from(10)).into(),
            UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::from("a")).into(),
            VarDeclList::from(vec![VarDecl::new("a", None)]).into(),
        ],
    );
}

#[test]
fn ambigous_regex_divide_expression() {
    let s = "1 / a === 1 / b";

    check_parser(
        s,
        vec![BinOp::new(
            CompOp::StrictEqual,
            BinOp::new(NumOp::Div, Const::Int(1), Identifier::from("a")),
            BinOp::new(NumOp::Div, Const::Int(1), Identifier::from("b")),
        )
        .into()],
    );
}

#[test]
fn two_divisions_in_expression() {
    let s = "a !== 0 || 1 / a === 1 / b;";

    check_parser(
        s,
        vec![BinOp::new(
            LogOp::Or,
            BinOp::new(CompOp::StrictNotEqual, Identifier::from("a"), Const::Int(0)),
            BinOp::new(
                CompOp::StrictEqual,
                BinOp::new(NumOp::Div, Const::Int(1), Identifier::from("a")),
                BinOp::new(NumOp::Div, Const::Int(1), Identifier::from("b")),
            ),
        )
        .into()],
    );
}

#[test]
fn comment_semi_colon_insertion() {
    let s = r#"
    let a = 10 // Comment
    let b = 20;
    "#;

    check_parser(
        s,
        vec![
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "a",
                Some(Const::Int(10).into()),
            )])
            .into(),
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "b",
                Some(Const::Int(20).into()),
            )])
            .into(),
        ],
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

    check_parser(
        s,
        vec![
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "a",
                Some(Const::Int(10).into()),
            )])
            .into(),
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "b",
                Some(Const::Int(20).into()),
            )])
            .into(),
        ],
    );
}

#[test]
fn multiline_comment_no_lineterminator() {
    let s = r#"
    let a = 10; /* Test comment */ let b = 20;
    "#;

    check_parser(
        s,
        vec![
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "a",
                Some(Const::Int(10).into()),
            )])
            .into(),
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "b",
                Some(Const::Int(20).into()),
            )])
            .into(),
        ],
    );
}

#[test]
fn assignment_line_terminator() {
    let s = r#"
    let a = 3;

    a =
    5;
    "#;

    check_parser(
        s,
        vec![
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "a",
                Some(Const::Int(3).into()),
            )])
            .into(),
            Assign::new(Identifier::from("a"), Const::from(5)).into(),
        ],
    );
}

#[test]
fn assignment_multiline_terminator() {
    let s = r#"
    let a = 3;


    a =


    5;
    "#;

    check_parser(
        s,
        vec![
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "a",
                Some(Const::Int(3).into()),
            )])
            .into(),
            Assign::new(Identifier::from("a"), Const::from(5)).into(),
        ],
    );
}

#[test]
fn bracketed_expr() {
    let s = r#"(b)"#;

    check_parser(s, vec![Identifier::from("b").into()]);
}

#[test]
fn increment_in_comma_op() {
    let s = r#"(b++, b)"#;

    check_parser(
        s,
        vec![BinOp::new::<_, Node, Node>(
            op::BinOp::Comma,
            UnaryOp::new::<Node>(op::UnaryOp::IncrementPost, Identifier::from("b").into()).into(),
            Identifier::from("b").into(),
        )
        .into()],
    );
}

#[test]
fn spread_in_arrow_function() {
    let s = r#"
    (...b) => {
        b
    }
    "#;

    check_parser(
        s,
        vec![
            ArrowFunctionDecl::new::<Box<[FormalParameter]>, StatementList>(
                Box::new([FormalParameter::new("b", None, true)]),
                vec![Identifier::from("b").into()].into(),
            )
            .into(),
        ],
    );
}

#[test]
fn empty_statement() {
    check_parser(
        r"
            ;;var a = 10;
            if(a) ;
        ",
        vec![
            Node::Empty
            ,VarDeclList::from(vec![VarDecl::new(
                "a",
                Node::from(Const::from(10)),
            )])
            .into(),
            Node::If(If::new::<_, _, Node, _>(Identifier::from("a"), Node::Empty, None))
        ],
    );
}
