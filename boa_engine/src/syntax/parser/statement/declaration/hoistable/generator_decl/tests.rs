use crate::syntax::{
    ast::{
        function::{FormalParameterList, Generator},
        statement::StatementList,
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn generator_function_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "function* gen() {}",
        vec![Generator::new(
            Some(interner.get_or_intern_static("gen", utf16!("gen")).into()),
            FormalParameterList::default(),
            StatementList::default(),
        )
        .into()],
        interner,
    );
}
