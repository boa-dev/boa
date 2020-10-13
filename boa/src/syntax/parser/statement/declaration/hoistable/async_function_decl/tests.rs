use crate::syntax::{ast::node::AsyncFunctionDecl, parser::tests::check_parser};

/// Async function declaration parsing.
#[test]
fn async_function_declaration() {
    check_parser(
        "async function hello() {}",
        vec![AsyncFunctionDecl::new(Box::from("hello"), vec![], vec![]).into()],
    );
}

/// Async function declaration parsing with keywords.
#[test]
fn async_function_declaration_keywords() {
    check_parser(
        "async function yield() {}",
        vec![AsyncFunctionDecl::new(Box::from("yield"), vec![], vec![]).into()],
    );

    check_parser(
        "async function await() {}",
        vec![AsyncFunctionDecl::new(Box::from("await"), vec![], vec![]).into()],
    );
}
