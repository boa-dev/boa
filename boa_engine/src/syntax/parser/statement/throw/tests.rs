use crate::syntax::{
    ast::{expression::literal::Literal, statement::Throw},
    parser::tests::check_parser,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn check_throw_parsing() {
    let mut interner = Interner::default();
    check_parser(
        "throw 'error';",
        vec![Throw::new(
            Literal::from(interner.get_or_intern_static("error", utf16!("error"))).into(),
        )
        .into()],
        interner,
    );
}
