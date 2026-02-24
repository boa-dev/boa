use crate::{Context, JsValue, TestAction, run_test_actions};
use boa_macros::js_str;
use boa_parser::Source;
use indoc::indoc;

use super::{is_registered_symbol, is_unique_symbol, is_well_known_symbol};
use crate::symbol::JsSymbol;

#[test]
fn call_symbol_and_check_return_type() {
    run_test_actions([TestAction::assert_with_op("Symbol()", |val, _| {
        val.is_symbol()
    })]);
}

#[test]
fn print_symbol_expect_description() {
    run_test_actions([TestAction::assert_eq(
        "String(Symbol('Hello'))",
        js_str!("Symbol(Hello)"),
    )]);
}

#[test]
fn symbol_access() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                var x = {};
                var sym1 = Symbol("Hello");
                var sym2 = Symbol("Hello");
                x[sym1] = 10;
                x[sym2] = 20;
            "#}),
        TestAction::assert_eq("x[sym1]", 10),
        TestAction::assert_eq("x[sym2]", 20),
        TestAction::assert_eq("x['Symbol(Hello)']", JsValue::undefined()),
    ]);
}

#[test]
fn test_is_registered_symbol() {
    let mut context = Context::default();

    // Symbol created via Symbol.for() should be registered
    let result = context
        .eval(Source::from_bytes(
            indoc! {r#"
            Symbol.for('test')
        "#}
            .as_bytes(),
        ))
        .unwrap();
    let registered_sym = result.as_symbol().unwrap();
    assert!(
        is_registered_symbol(&registered_sym),
        "Symbol created via Symbol.for() should be registered"
    );

    // Symbol created via Symbol() should NOT be registered
    let result = context
        .eval(Source::from_bytes(
            indoc! {r#"
            Symbol('test')
        "#}
            .as_bytes(),
        ))
        .unwrap();
    let unique_sym = result.as_symbol().unwrap();
    assert!(
        !is_registered_symbol(&unique_sym),
        "Symbol created via Symbol() should not be registered"
    );

    // Well-known symbols should NOT be registered
    let well_known_sym = JsSymbol::iterator();
    assert!(
        !is_registered_symbol(&well_known_sym),
        "Well-known symbols should not be registered"
    );
}

#[test]
fn test_is_well_known_symbol() {
    // Test all well-known symbols
    let well_known_symbols = [
        JsSymbol::async_iterator(),
        JsSymbol::has_instance(),
        JsSymbol::is_concat_spreadable(),
        JsSymbol::iterator(),
        JsSymbol::r#match(),
        JsSymbol::match_all(),
        JsSymbol::replace(),
        JsSymbol::search(),
        JsSymbol::species(),
        JsSymbol::split(),
        JsSymbol::to_primitive(),
        JsSymbol::to_string_tag(),
        JsSymbol::unscopables(),
    ];

    for (i, sym) in well_known_symbols.iter().enumerate() {
        assert!(
            is_well_known_symbol(sym),
            "Well-known symbol at index {} should be recognized",
            i
        );
    }

    // Regular symbol should NOT be well-known
    let regular_sym = JsSymbol::new(Some(js_str!("test").into())).unwrap();
    assert!(
        !is_well_known_symbol(&regular_sym),
        "Regular symbol should not be well-known"
    );

    // Registered symbol should NOT be well-known
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            indoc! {r#"Symbol.for('test')"#}.as_bytes(),
        ))
        .unwrap();
    let registered_sym = result.as_symbol().unwrap();
    assert!(
        !is_well_known_symbol(&registered_sym),
        "Registered symbol should not be well-known"
    );
}

#[test]
fn test_is_unique_symbol() {
    // Symbol created via Symbol() should be unique
    let unique_sym = JsSymbol::new(Some(js_str!("test").into())).unwrap();
    assert!(
        is_unique_symbol(&unique_sym),
        "Symbol created via Symbol() should be unique"
    );

    // Symbol created via Symbol() without description should be unique
    let unique_sym_no_desc = JsSymbol::new(None).unwrap();
    assert!(
        is_unique_symbol(&unique_sym_no_desc),
        "Symbol created via Symbol() without description should be unique"
    );

    // Registered symbol should NOT be unique
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            indoc! {r#"Symbol.for('test')"#}.as_bytes(),
        ))
        .unwrap();
    let registered_sym = result.as_symbol().unwrap();
    assert!(
        !is_unique_symbol(&registered_sym),
        "Registered symbol should not be unique"
    );

    // Well-known symbols should NOT be unique
    let well_known_symbols = [
        JsSymbol::async_iterator(),
        JsSymbol::has_instance(),
        JsSymbol::is_concat_spreadable(),
        JsSymbol::iterator(),
        JsSymbol::r#match(),
        JsSymbol::match_all(),
        JsSymbol::replace(),
        JsSymbol::search(),
        JsSymbol::species(),
        JsSymbol::split(),
        JsSymbol::to_primitive(),
        JsSymbol::to_string_tag(),
        JsSymbol::unscopables(),
    ];

    for (i, sym) in well_known_symbols.iter().enumerate() {
        assert!(
            !is_unique_symbol(sym),
            "Well-known symbol at index {} should not be unique",
            i
        );
    }
}
