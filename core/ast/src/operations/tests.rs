use boa_interner::Interner;

use crate::{
    Span, Statement,
    expression::{Call, Identifier, NewTarget, This},
    operations::{ContainsSymbol, contains},
    statement::With,
};

#[test]
fn check_contains_this_in_with_statment_expression() {
    let node = With::new(This::new(Span::EMPTY).into(), Statement::Empty);
    assert!(contains(&node, ContainsSymbol::This));
}

#[test]
fn check_contains_new_target_in_with_statment_expression() {
    let node = With::new(NewTarget::new(Span::EMPTY).into(), Statement::Empty);
    assert!(contains(&node, ContainsSymbol::NewTarget));
}

#[test]
fn check_contains_new_target_in_call_function_position() {
    let node = Call::new(
        NewTarget::new(Span::EMPTY).into(),
        Box::default(),
        Span::EMPTY,
    );
    assert!(contains(&node, ContainsSymbol::NewTarget));
}

#[test]
fn check_contains_this_in_call_argument_position() {
    let mut interner = Interner::new();
    let function_name = Identifier::new(interner.get_or_intern("func"), Span::new((1, 1), (1, 5)));
    let node = Call::new(
        function_name.into(),
        vec![This::new(Span::EMPTY).into()].into_boxed_slice(),
        Span::EMPTY,
    );

    assert!(contains(&node, ContainsSymbol::This));
}

#[test]
fn check_contains_new_target_in_call_argument_position() {
    let mut interner = Interner::new();
    let function_name = Identifier::new(interner.get_or_intern("func"), Span::new((1, 1), (1, 5)));
    let node = Call::new(
        function_name.into(),
        vec![NewTarget::new(Span::EMPTY).into()].into_boxed_slice(),
        Span::EMPTY,
    );

    assert!(contains(&node, ContainsSymbol::NewTarget));
}
