use crate::parser::tests::check_script_parser;
use boa_ast::{
    function::{FormalParameterList, FunctionBody, Generator},
    Declaration,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn generator_function_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "function* gen() {}",
        vec![Declaration::Generator(Generator::new(
            Some(interner.get_or_intern_static("gen", utf16!("gen")).into()),
            FormalParameterList::default(),
            FunctionBody::default(),
            false,
        ))
        .into()],
        interner,
    );
}
