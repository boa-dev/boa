use crate::parser::tests::check_parser;
use boa_ast::{
    function::{FormalParameterList, Generator},
    Declaration, StatementList,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn generator_function_declaration() {
    let interner = &mut Interner::default();
    check_parser(
        "function* gen() {}",
        vec![Declaration::Generator(Generator::new(
            Some(interner.get_or_intern_static("gen", utf16!("gen")).into()),
            FormalParameterList::default(),
            StatementList::default(),
            false,
        ))
        .into()],
        interner,
    );
}
