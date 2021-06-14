use crate::syntax::{
    ast::node::{BinOp, ClassDecl, Declaration, DeclarationList, FunctionDecl, Identifier},
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
                let val;
            }
        }
        "#,
        vec![ClassDecl::new(
            Box::from("Basic"),
            vec![FunctionDecl::new(
                Box::from("constructor"),
                vec![],
                vec![
                    DeclarationList::Let(vec![Declaration::new("val", None)].into_boxed_slice())
                        .into(),
                ],
            )],
        )
        .into()],
    );
}
