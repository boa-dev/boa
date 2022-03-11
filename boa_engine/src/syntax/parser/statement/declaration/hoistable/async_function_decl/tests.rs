use crate::syntax::{
    ast::node::{AsyncFunctionDecl, FormalParameterList},
    parser::tests::check_parser,
};
use boa_interner::Interner;

/// Async function declaration parsing.
#[test]
fn async_function_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "async function hello() {}",
        vec![AsyncFunctionDecl::new(
            interner.get_or_intern_static("hello"),
            FormalParameterList::default(),
            vec![],
        )
        .into()],
        &mut interner,
    );
}

/// Async function declaration parsing with keywords.
#[test]
fn async_function_declaration_keywords() {
    let mut interner = Interner::default();
    check_parser(
        "async function yield() {}",
        vec![AsyncFunctionDecl::new(
            interner.get_or_intern_static("yield"),
            FormalParameterList::default(),
            vec![],
        )
        .into()],
        &mut interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "async function await() {}",
        vec![AsyncFunctionDecl::new(
            interner.get_or_intern_static("await"),
            FormalParameterList::default(),
            vec![],
        )
        .into()],
        &mut interner,
    );
}
