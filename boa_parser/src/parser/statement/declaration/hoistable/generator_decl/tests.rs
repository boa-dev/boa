use crate::parser::tests::check_script_parser;
use boa_ast::{
    function::{FormalParameterList, FunctionBody, Generator},
    Declaration,
};
use boa_interner::Interner;

#[test]
fn generator_function_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "function* gen() {}",
        vec![Declaration::Generator(Generator::new(
            Some(interner.get_or_intern("gen").into()),
            FormalParameterList::default(),
            FunctionBody::default(),
            false,
        ))
        .into()],
        interner,
    );
}
