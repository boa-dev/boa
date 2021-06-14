use crate::syntax::{
    ast::node::{BinOp, ClassDecl, FunctionDecl, Identifier, Return},
    ast::op::NumOp,
    parser::tests::check_parser,
};

/// Checks for empty class parsing (making sure the keyword works)
#[test]
fn check_empty() {
    check_parser(
        "class Empty {}",
        vec![ClassDecl::new(Box::from("Empty"), vec![]).into()],
    );
}

/// Checks for a constructor being parsed in a class
#[test]
fn check_basic() {
    check_parser(
        r#"
        class Basic {
            constructor() {
                console.log("Hello, world!");
            }
        }
        "#,
        vec![ClassDecl::new(
            Box::from("Basic"),
            vec![FunctionDecl::new(
                Box::from("constructor"),
                vec![],
                vec![Return::new(
                    BinOp::new(NumOp::Add, Identifier::from("a"), Identifier::from("b")),
                    None,
                )
                .into()],
            )],
        )
        .into()],
    );
}
