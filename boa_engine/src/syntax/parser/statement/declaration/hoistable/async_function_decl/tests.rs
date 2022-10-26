use crate::syntax::{
    ast::{
        function::{AsyncFunction, FormalParameterList},
        Declaration, StatementList,
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Async function declaration parsing.
#[test]
fn async_function_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "async function hello() {}",
        vec![Declaration::AsyncFunction(AsyncFunction::new(
            Some(
                interner
                    .get_or_intern_static("hello", utf16!("hello"))
                    .into(),
            ),
            FormalParameterList::default(),
            StatementList::default(),
        ))
        .into()],
        interner,
    );
}

/// Async function declaration parsing with keywords.
#[test]
fn async_function_declaration_keywords() {
    let mut interner = Interner::default();
    check_parser(
        "async function yield() {}",
        vec![Declaration::AsyncFunction(AsyncFunction::new(
            Some(
                interner
                    .get_or_intern_static("yield", utf16!("yield"))
                    .into(),
            ),
            FormalParameterList::default(),
            StatementList::default(),
        ))
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "async function await() {}",
        vec![Declaration::AsyncFunction(AsyncFunction::new(
            Some(
                interner
                    .get_or_intern_static("await", utf16!("await"))
                    .into(),
            ),
            FormalParameterList::default(),
            StatementList::default(),
        ))
        .into()],
        interner,
    );
}
