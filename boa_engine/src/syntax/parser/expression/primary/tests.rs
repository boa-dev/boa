use crate::syntax::{
    ast::{expression::literal::Literal, Expression},
    parser::tests::check_parser,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;

#[test]
fn check_string() {
    // Check empty string
    check_parser(
        "\"\"",
        vec![Expression::from(Literal::from(Sym::EMPTY_STRING)).into()],
        Interner::default(),
    );

    // Check non-empty string
    let mut interner = Interner::default();
    check_parser(
        "\"hello\"",
        vec![Expression::from(Literal::from(
            interner.get_or_intern_static("hello", utf16!("hello")),
        ))
        .into()],
        interner,
    );
}
