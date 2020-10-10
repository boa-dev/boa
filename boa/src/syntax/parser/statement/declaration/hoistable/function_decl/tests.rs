use crate::syntax::{ast::node::FunctionDecl, parser::tests::check_parser};

/// Function declaration parsing.
#[test]
fn function_declaration() {
    check_parser(
        "function hello() {}",
        vec![FunctionDecl::new(Box::from("hello"), vec![], vec![]).into()],
    );
}

/// Function declaration parsing with keywords.
#[test]
fn function_declaration_keywords() {
    check_parser(
        "function yield() {}",
        vec![FunctionDecl::new(Box::from("yield"), vec![], vec![]).into()],
    );

    check_parser(
        "function await() {}",
        vec![FunctionDecl::new(Box::from("await"), vec![], vec![]).into()],
    );
}
