use crate::{
    syntax::{ast::node::GeneratorDecl, parser::tests::check_parser},
    Interner,
};

#[test]
fn generator_function_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "function* gen() {}",
        vec![GeneratorDecl::new(Box::from("gen"), vec![], vec![]).into()],
        &mut interner,
    );
}
