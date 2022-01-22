use crate::{
    syntax::{ast::node::AsyncGeneratorDecl, parser::tests::check_parser},
    Interner,
};

#[test]
fn async_generator_function_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "async function* gen() {}",
        vec![AsyncGeneratorDecl::new(Box::from("gen"), vec![], vec![]).into()],
        &mut interner,
    );
}
