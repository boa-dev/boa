//! Tests for the parser.

use super::Parser;
use crate::syntax::ast::{
    node::{
        field::GetConstField, Assign, BinOp, Call, FunctionDecl, Identifier, New, Node, Return,
        StatementList, UnaryOp, VarDecl, VarDeclList,
    },
    op::{self, NumOp},
    Const,
};

/// Checks that the given JavaScript string gives the expected expression.
#[allow(clippy::result_unwrap_used)]
// TODO: #[track_caller]: https://github.com/rust-lang/rust/issues/47809
pub(super) fn check_parser<L>(js: &str, expr: L)
where
    L: Into<Box<[Node]>>,
{
    assert_eq!(
        Parser::new(js.as_bytes())
            .parse_all()
            .expect("failed to parse"),
        StatementList::from(expr)
    );
}

/// Checks that the given javascript string creates a parse error.
// TODO: #[track_caller]: https://github.com/rust-lang/rust/issues/47809
pub(super) fn check_invalid(js: &str) {
    assert!(Parser::new(js.as_bytes()).parse_all().is_err());
}

//
// a[0].hi = 'hi';
// a[0].hi
//

#[test]
fn array_fill_obj_ref() {
    let init = r#"a = (new Array(3)).fill({});"#;
    let res = Parser::new(init.as_bytes())
        .parse_all()
        .expect("failed to parse");

    println!("Result {:?}", res);
}

// #[test]
// fn check_single_statement() {
//     let init = r#"
//         arr.pop();
//         "#;
//     let res = Parser::new(init.as_bytes())
//     .parse_all()
//     .expect("failed to parse");

//     println!("Result {:?}", res);
// }

// // #[test]
// // fn check_no_semicolon_statement() {
// //     let init = r#"
// //         arr.pop();
// //         return arr.len();
// //         "#;
// //     let res = Parser::new(init.as_bytes())
// //     .parse_all()
// //     .expect("failed to parse");
// //
// //     println!("Result {:?}", res);
// // }

// #[test]
// fn check_function_no_semicolon_statement() {
//     let init = r#"
//         var arr = [11, 23, 45];

//         function foo() {
//           arr.pop()
//           return arr.len < 3;
//         }
//         "#;
//     let res = Parser::new(init.as_bytes())
//     .parse_all()
//     .expect("failed to parse");

//     println!("Result {:?}", res);
// }

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
                vec![Return::new(Const::from(10)).into()],
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
