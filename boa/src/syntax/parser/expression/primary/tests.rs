use crate::syntax::{ast::Const, parser::tests::check_parser};

#[test]
fn check_string() {
    // Check empty string
    check_parser("\"\"", vec![Const::from("").into()]);

    // Check non-empty string
    check_parser("\"hello\"", vec![Const::from("hello").into()]);
}
