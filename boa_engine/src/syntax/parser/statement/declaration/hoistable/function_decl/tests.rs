use crate::syntax::{
    ast::node::{FormalParameterList, FunctionDecl},
    parser::tests::check_parser,
};
use boa_interner::Interner;

/// Function declaration parsing.
#[test]
fn function_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "function hello() {}",
        vec![FunctionDecl::new(
            interner.get_or_intern_static("hello"),
            FormalParameterList::default(),
            vec![],
        )
        .into()],
        interner,
    );
}

/// Function declaration parsing with keywords.
#[test]
fn function_declaration_keywords() {
    let genast = |keyword, interner: &mut Interner| {
        vec![FunctionDecl::new(
            interner.get_or_intern_static(keyword),
            FormalParameterList::default(),
            vec![],
        )
        .into()]
    };

    let mut interner = Interner::default();
    let ast = genast("yield", &mut interner);
    check_parser("function yield() {}", ast, interner);

    let mut interner = Interner::default();
    let ast = genast("await", &mut interner);
    check_parser("function await() {}", ast, interner);

    let mut interner = Interner::default();
    let ast = genast("as", &mut interner);
    check_parser("function as() {}", ast, interner);

    let mut interner = Interner::default();
    let ast = genast("async", &mut interner);
    check_parser("function async() {}", ast, interner);

    let mut interner = Interner::default();
    let ast = genast("from", &mut interner);
    check_parser("function from() {}", ast, interner);

    let mut interner = Interner::default();
    let ast = genast("get", &mut interner);
    check_parser("function get() {}", ast, interner);

    let mut interner = Interner::default();
    let ast = genast("meta", &mut interner);
    check_parser("function meta() {}", ast, interner);

    let mut interner = Interner::default();
    let ast = genast("of", &mut interner);
    check_parser("function of() {}", ast, interner);

    let mut interner = Interner::default();
    let ast = genast("set", &mut interner);
    check_parser("function set() {}", ast, interner);

    let mut interner = Interner::default();
    let ast = genast("target", &mut interner);
    check_parser("function target() {}", ast, interner);
}
