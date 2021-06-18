use crate::syntax::{
    ast::node::{ClassDecl, Declaration, DeclarationList, FunctionDecl},
    parser::{class::ClassField, tests::check_parser, Parser},
};

/// Checks for empty class parsing (making sure the keyword works)
#[test]
fn check_empty() {
    check_parser(
        "class Empty {}",
        vec![ClassDecl::new(Box::from("Empty"), None, vec![], vec![]).into()],
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
            Some(FunctionDecl::new(
                Box::from("constructor"),
                vec![],
                vec![
                    DeclarationList::Let(vec![Declaration::new("val", None)].into_boxed_slice())
                        .into(),
                ],
            )),
            vec![],
            vec![],
        )
        .into()],
    );
}

/// Checks for a static method being parsed in a class
#[test]
fn check_static() {
    check_parser(
        r#"
        class Basic {
            static say_hello() {
                let val;
            }
        }
        "#,
        vec![ClassDecl::new(
            Box::from("Basic"),
            None,
            vec![],
            vec![ClassField::Method(FunctionDecl::new(
                Box::from("say_hello"),
                vec![],
                vec![
                    DeclarationList::Let(vec![Declaration::new("val", None)].into_boxed_slice())
                        .into(),
                ],
            ))],
        )
        .into()],
    );
}

/// Checks for multiple functions being parsed.
#[test]
fn check_multi() {
    check_parser(
        r#"
        class Multi {
            constructor() {
                let val;
            }
            say_hello() {}
            say_hello_again() {}
        }
        "#,
        vec![ClassDecl::new(
            Box::from("Multi"),
            Some(FunctionDecl::new(
                Box::from("constructor"),
                vec![],
                vec![
                    DeclarationList::Let(vec![Declaration::new("val", None)].into_boxed_slice())
                        .into(),
                ],
            )),
            vec![
                ClassField::Method(FunctionDecl::new(Box::from("say_hello"), vec![], vec![])),
                ClassField::Method(FunctionDecl::new(
                    Box::from("say_hello_again"),
                    vec![],
                    vec![],
                )),
            ],
            vec![],
        )
        .into()],
    );
}

/// Checks for multiple constructors being a parse error.
#[test]
fn check_multi_constructors() {
    let js = r#"
        class InvalidBecauseConstructors {
            constructor() {
            }
            constructor() {
            }
        }
        "#;
    assert!(Parser::new(js.as_bytes(), false).parse_all().is_err());
}
