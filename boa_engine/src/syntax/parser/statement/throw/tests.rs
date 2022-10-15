use crate::{
    string::utf16,
    syntax::{
        ast::{node::Throw, Const},
        parser::tests::check_parser,
    },
};
use boa_interner::Interner;

#[test]
fn check_throw_parsing() {
    let mut interner = Interner::default();
    check_parser(
        "throw 'error';",
        vec![Throw::new(Const::from(
            interner.get_or_intern_static("error", utf16!("error")),
        ))
        .into()],
        interner,
    );
}
