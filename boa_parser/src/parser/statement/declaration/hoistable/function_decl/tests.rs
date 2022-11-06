use crate::parser::tests::check_parser;
use boa_ast::{
    function::{FormalParameterList, Function},
    Declaration, StatementList,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Function declaration parsing.
#[test]
fn function_declaration() {
    let interner = &mut Interner::default();
    check_parser(
        "function hello() {}",
        vec![Declaration::Function(Function::new(
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

/// Function declaration parsing with keywords.
#[test]
fn function_declaration_keywords() {
    macro_rules! genast {
        ($keyword:literal, $interner:expr) => {
            vec![Declaration::Function(Function::new(
                Some(
                    $interner
                        .get_or_intern_static($keyword, utf16!($keyword))
                        .into(),
                ),
                FormalParameterList::default(),
                StatementList::default(),
            ))
            .into()]
        };
    }

    let interner = &mut Interner::default();
    let ast = genast!("yield", interner);
    check_parser("function yield() {}", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("await", interner);
    check_parser("function await() {}", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("as", interner);
    check_parser("function as() {}", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("async", interner);
    check_parser("function async() {}", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("from", interner);
    check_parser("function from() {}", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("get", interner);
    check_parser("function get() {}", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("meta", interner);
    check_parser("function meta() {}", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("of", interner);
    check_parser("function of() {}", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("set", interner);
    check_parser("function set() {}", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("target", interner);
    check_parser("function target() {}", ast, interner);
}
