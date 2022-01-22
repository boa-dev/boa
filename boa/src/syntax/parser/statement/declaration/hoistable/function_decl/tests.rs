use crate::{
    syntax::{ast::node::FunctionDecl, parser::tests::check_parser},
    Interner,
};

/// Function declaration parsing.
#[test]
fn function_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "function hello() {}",
        vec![FunctionDecl::new(Box::from("hello"), vec![], vec![]).into()],
        &mut interner,
    );
}

/// Function declaration parsing with keywords.
#[test]
fn function_declaration_keywords() {
    let mut interner = Interner::new();
    check_parser(
        "function yield() {}",
        vec![FunctionDecl::new(Box::from("yield"), vec![], vec![]).into()],
        &mut interner,
    );

    let mut interner = Interner::new();
    check_parser(
        "function await() {}",
        vec![FunctionDecl::new(Box::from("await"), vec![], vec![]).into()],
        &mut interner,
    );
}
