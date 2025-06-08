use boa_interner::Interner;

use crate::{
    expression::{Call, Identifier, This},
    operations::{contains, ContainsSymbol},
    statement::With,
    Expression, Span, Statement,
};

#[test]
fn check_contains_this_in_with_statment_expression() {
    let node = With::new(
        This::new(Span::new((1, 1), (1, 1))).into(),
        Statement::Empty,
    );
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
    let function_name = Identifier::new(interner.get_or_intern("func"), Span::new((1, 1), (1, 5)));
    let node = Call::new(
        function_name.into(),
        vec![This::new(Span::new((1, 1), (1, 1))).into()].into_boxed_slice(),
    );

    assert!(contains(&node, ContainsSymbol::This));
}

#[test]
fn check_contains_new_target_in_call_argument_position() {
    let mut interner = Interner::new();
    let function_name = Identifier::new(interner.get_or_intern("func"), Span::new((1, 1), (1, 5)));
    let node = Call::new(
        function_name.into(),
        vec![Expression::NewTarget].into_boxed_slice(),
    );

    assert!(contains(&node, ContainsSymbol::NewTarget));
}
