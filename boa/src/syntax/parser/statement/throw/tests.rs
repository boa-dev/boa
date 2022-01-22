use crate::{
    syntax::{
        ast::{node::Throw, Const},
        parser::tests::check_parser,
    },
    Interner,
};

#[test]
fn check_throw_parsing() {
    let mut interner = Interner::new();
    check_parser(
        "throw 'error';",
        vec![Throw::new(Const::from("error")).into()],
        &mut interner,
    );
}
