use crate::parser::tests::check_parser;
use boa_ast::{expression::literal::Literal, Expression, Statement};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;

#[test]
fn check_string() {
    // Check empty string
    check_parser(
        "\"\"",
        vec![Statement::Expression(Expression::from(Literal::from(Sym::EMPTY_STRING))).into()],
        &mut Interner::default(),
    );

    // Check non-empty string
    let interner = &mut Interner::default();
    check_parser(
        "\"hello\"",
        vec![Statement::Expression(Expression::from(Literal::from(
            interner.get_or_intern_static("hello", utf16!("hello")),
        )))
        .into()],
        interner,
    );
}
