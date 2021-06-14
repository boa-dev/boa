use crate::syntax::{ast::node::FunctionDecl, parser::tests::check_parser};

/// Function declaration parsing.
#[test]
fn function_declaration() {
    check_parser(
        "class empty {}",
        vec![FunctionDecl::new(Box::from("empty"), vec![], vec![]).into()],
    );
}
