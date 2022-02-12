use crate::{
    syntax::{ast::node::FunctionDecl, parser::tests::check_parser},
    Interner,
};

/// Function declaration parsing.
#[test]
fn function_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "function hello() {}",
        vec![FunctionDecl::new(interner.get_or_intern_static("hello"), vec![], vec![]).into()],
        &mut interner,
    );
}

/// Function declaration parsing with keywords.
#[test]
fn function_declaration_keywords() {
    let mut interner = Interner::default();
    check_parser(
        "function yield() {}",
        vec![FunctionDecl::new(interner.get_or_intern_static("yield"), vec![], vec![]).into()],
        &mut interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "function await() {}",
        vec![FunctionDecl::new(interner.get_or_intern_static("await"), vec![], vec![]).into()],
        &mut interner,
    );
}
