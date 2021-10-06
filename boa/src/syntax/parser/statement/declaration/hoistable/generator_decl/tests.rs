use crate::syntax::{ast::node::GeneratorDecl, parser::tests::check_parser};

#[test]
fn generator_function_declaration() {
    check_parser(
        "function* gen() {}",
        vec![GeneratorDecl::new(Box::from("gen"), vec![], vec![]).into()],
    );
}
