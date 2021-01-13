use crate::syntax::{
    ast::node::{FormalParameter, FunctionDecl, Identifier, Node, Return},
    parser::tests::check_parser,
};

/// Checks basic function declaration parsing.
#[test]
fn check_basic() {
    check_parser(
        "function foo(a) { return a; }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new("a", None, false)],
            vec![Return::new(Identifier::from("a"), None).into()],
        )
        .into()],
    );
}

/// Checks basic function declaration parsing with automatic semicolon insertion.
#[test]
fn check_basic_semicolon_insertion() {
    check_parser(
        "function foo(a) { return a }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new("a", None, false)],
            vec![Return::new(Identifier::from("a"), None).into()],
        )
        .into()],
    );
}

/// Checks functions with empty returns.
#[test]
fn check_empty_return() {
    check_parser(
        "function foo(a) { return; }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new("a", None, false)],
            vec![Return::new::<Node, Option<Node>, Option<_>>(None, None).into()],
        )
        .into()],
    );
}

/// Checks functions with empty returns without semicolon
#[test]
fn check_empty_return_semicolon_insertion() {
    check_parser(
        "function foo(a) { return }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new("a", None, false)],
            vec![Return::new::<Node, Option<Node>, Option<_>>(None, None).into()],
        )
        .into()],
    );
}

/// Checks rest operator parsing.
#[test]
fn check_rest_operator() {
    check_parser(
        "function foo(a, ...b) {}",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, true),
            ],
            vec![],
        )
        .into()],
    );
}
