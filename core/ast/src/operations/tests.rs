use boa_interner::Interner;

use crate::{
    expression::{Call, Identifier},
    operations::{contains, ContainsSymbol},
    statement::With,
    Expression, Statement,
};

#[test]
fn check_contains_this_in_with_statment_expression() {
    let node = With::new(Expression::This, Statement::Empty);
    assert!(contains(&node, ContainsSymbol::This));
}

#[test]
fn check_contains_new_target_in_with_statment_expression() {
    let node = With::new(Expression::NewTarget, Statement::Empty);
    assert!(contains(&node, ContainsSymbol::NewTarget));
}

#[test]
fn check_contains_new_target_in_call_function_position() {
    let node = Call::new(Expression::NewTarget, Box::default());
    assert!(contains(&node, ContainsSymbol::NewTarget));
}

#[test]
fn check_contains_this_in_call_argument_position() {
    let mut interner = Interner::new();
    let function_name: Identifier = interner.get_or_intern("func").into();
    let node = Call::new(
        function_name.into(),
        vec![Expression::This].into_boxed_slice(),
    );

    assert!(contains(&node, ContainsSymbol::This));
}

#[test]
fn check_contains_new_target_in_call_argument_position() {
    let mut interner = Interner::new();
    let function_name: Identifier = interner.get_or_intern("func").into();
    let node = Call::new(
        function_name.into(),
        vec![Expression::NewTarget].into_boxed_slice(),
    );

    assert!(contains(&node, ContainsSymbol::NewTarget));
}
