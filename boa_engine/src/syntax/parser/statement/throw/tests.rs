use crate::syntax::{
    ast::{expression::literal::Literal, statement::Throw, Statement},
    parser::tests::check_parser,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn check_throw_parsing() {
    let mut interner = Interner::default();
    check_parser(
        "throw 'error';",
        vec![Statement::Throw(Throw::new(
            Literal::from(interner.get_or_intern_static("error", utf16!("error"))).into(),
        ))
        .into()],
        interner,
    );
}
