use crate::syntax::{ast::node::ClassDecl, parser::tests::check_parser};

/// Function declaration parsing.
#[test]
fn function_declaration() {
    check_parser(
        "class empty {}",
        vec![ClassDecl::new(Box::from("empty"), vec![]).into()],
    );
}
