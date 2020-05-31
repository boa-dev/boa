use crate::syntax::{
    ast::{node::Throw, Const},
    parser::tests::check_parser,
};

#[test]
fn check_throw_parsing() {
    check_parser(
        "throw 'error';",
        vec![Throw::new(Const::from("error")).into()],
    );
}
