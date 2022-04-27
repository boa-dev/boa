use crate::syntax::{
    ast::node::{AsyncGeneratorDecl, FormalParameterList},
    parser::tests::check_parser,
};
use boa_interner::Interner;

#[test]
fn async_generator_function_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "async function* gen() {}",
        vec![AsyncGeneratorDecl::new(
            interner.get_or_intern_static("gen"),
            FormalParameterList::default(),
            vec![],
        )
        .into()],
        interner,
    );
}
