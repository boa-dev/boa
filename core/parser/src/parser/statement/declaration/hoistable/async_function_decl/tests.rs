use crate::parser::tests::check_script_parser;
use boa_ast::{
    function::{AsyncFunctionDeclaration, FormalParameterList, FunctionBody},
    Declaration,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;

/// Async function declaration parsing.
#[test]
fn async_function_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "async function hello() {}",
        vec![
            Declaration::AsyncFunctionDeclaration(AsyncFunctionDeclaration::new(
                interner
                    .get_or_intern_static("hello", utf16!("hello"))
                    .into(),
                FormalParameterList::default(),
                FunctionBody::default(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Async function declaration parsing with keywords.
#[test]
fn async_function_declaration_keywords() {
    let interner = &mut Interner::default();
    check_script_parser(
        "async function yield() {}",
        vec![
            Declaration::AsyncFunctionDeclaration(AsyncFunctionDeclaration::new(
                Sym::YIELD.into(),
                FormalParameterList::default(),
                FunctionBody::default(),
            ))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "async function await() {}",
        vec![
            Declaration::AsyncFunctionDeclaration(AsyncFunctionDeclaration::new(
                Sym::AWAIT.into(),
                FormalParameterList::default(),
                FunctionBody::default(),
            ))
            .into(),
        ],
        interner,
    );
}
