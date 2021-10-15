use crate::syntax::{ast::node::AsyncGeneratorDecl, parser::tests::check_parser};

#[test]
fn async_generator_function_declaration() {
    check_parser(
        "async function* gen() {}",
        vec![AsyncGeneratorDecl::new(Box::from("gen"), vec![], vec![]).into()],
    );
}