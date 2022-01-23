use crate::{
    syntax::{ast::node::AsyncGeneratorDecl, parser::tests::check_parser},
    Interner,
};

#[test]
fn async_generator_function_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "async function* gen() {}",
        vec![AsyncGeneratorDecl::new(interner.get_or_intern_static("gen"), vec![], vec![]).into()],
        &mut interner,
    );
}
