use crate::parser::tests::check_script_parser;
use boa_ast::{
    function::{AsyncGenerator, FormalParameterList, FunctionBody},
    Declaration,
};
use boa_interner::Interner;

#[test]
fn async_generator_function_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "async function* gen() {}",
        vec![Declaration::AsyncGenerator(AsyncGenerator::new(
            Some(interner.get_or_intern("gen").into()),
            FormalParameterList::default(),
            FunctionBody::default(),
            false,
        ))
        .into()],
        interner,
    );
}
