use crate::syntax::{
    ast::{Const, Node},
    parser::tests::check_parser,
};

#[test]
fn check_throw_parsing() {
    check_parser("throw 'error';", vec![Node::throw(Const::from("error"))]);
}
