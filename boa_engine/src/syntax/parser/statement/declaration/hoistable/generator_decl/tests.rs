use crate::{
    string::utf16,
    syntax::{
        ast::node::{FormalParameterList, GeneratorDecl},
        parser::tests::check_parser,
    },
};
use boa_interner::Interner;

#[test]
fn generator_function_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "function* gen() {}",
        vec![GeneratorDecl::new(
            interner.get_or_intern_static("gen", utf16!("gen")),
            FormalParameterList::default(),
            vec![],
        )
        .into()],
        interner,
    );
}
