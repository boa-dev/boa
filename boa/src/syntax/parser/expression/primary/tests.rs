use crate::{
    syntax::{ast::Const, parser::tests::check_parser},
    Interner,
};

#[test]
fn check_string() {
    // Check empty string
    let mut interner = Interner::new();
    check_parser("\"\"", vec![Const::from("").into()], &mut interner);

    // Check non-empty string
    let mut interner = Interner::new();
    check_parser(
        "\"hello\"",
        vec![Const::from("hello").into()],
        &mut interner,
    );
}
