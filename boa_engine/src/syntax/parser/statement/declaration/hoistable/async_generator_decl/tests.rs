use crate::syntax::{
    ast::{
        function::{AsyncGenerator, FormalParameterList},
        statement::StatementList,
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn async_generator_function_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "async function* gen() {}",
        vec![AsyncGenerator::new(
            Some(interner.get_or_intern_static("gen", utf16!("gen"))),
            FormalParameterList::default(),
            StatementList::default(),
        )
        .into()],
        interner,
    );
}
