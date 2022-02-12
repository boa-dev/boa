use crate::{
    syntax::{ast::node::GeneratorDecl, parser::tests::check_parser},
    Interner,
};

#[test]
fn generator_function_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "function* gen() {}",
        vec![GeneratorDecl::new(interner.get_or_intern_static("gen"), vec![], vec![]).into()],
        &mut interner,
    );
}
