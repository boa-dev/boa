use crate::{
    syntax::{ast::node::AsyncFunctionDecl, parser::tests::check_parser},
    Interner,
};

/// Async function declaration parsing.
#[test]
fn async_function_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "async function hello() {}",
        vec![AsyncFunctionDecl::new(Box::from("hello"), vec![], vec![]).into()],
        &mut interner,
    );
}

/// Async function declaration parsing with keywords.
#[test]
fn async_function_declaration_keywords() {
    let mut interner = Interner::new();
    check_parser(
        "async function yield() {}",
        vec![AsyncFunctionDecl::new(Box::from("yield"), vec![], vec![]).into()],
        &mut interner,
    );

    let mut interner = Interner::new();
    check_parser(
        "async function await() {}",
        vec![AsyncFunctionDecl::new(Box::from("await"), vec![], vec![]).into()],
        &mut interner,
    );
}
