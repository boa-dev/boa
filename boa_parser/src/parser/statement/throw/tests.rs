use crate::parser::tests::check_parser;
use boa_ast::{expression::literal::Literal, statement::Throw, Statement};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn check_throw_parsing() {
    let interner = &mut Interner::default();
    check_parser(
        "throw 'error';",
        vec![Statement::Throw(Throw::new(
            Literal::from(interner.get_or_intern_static("error", utf16!("error"))).into(),
        ))
        .into()],
        interner,
    );
}
