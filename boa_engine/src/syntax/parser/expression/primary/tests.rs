use crate::{
    string::utf16,
    syntax::{ast::Const, parser::tests::check_parser},
};
use boa_interner::{Interner, Sym};

#[test]
fn check_string() {
    // Check empty string
    check_parser(
        "\"\"",
        vec![Const::from(Sym::EMPTY_STRING).into()],
        Interner::default(),
    );

    // Check non-empty string
    let mut interner = Interner::default();
    check_parser(
        "\"hello\"",
        vec![Const::from(interner.get_or_intern_static("hello", utf16!("hello"))).into()],
        interner,
    );
}
