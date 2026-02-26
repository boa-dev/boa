use crate::{Context, JsValue, TestAction, run_test_actions};
use boa_macros::js_str;
use boa_parser::Source;
use indoc::indoc;

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
        registered_sym.is_registered(),
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
        !unique_sym.is_registered(),
        "Symbol created via Symbol() should not be registered"
    );

    // Well-known symbols should NOT be registered
    let well_known_sym = JsSymbol::iterator();
    assert!(
        !well_known_sym.is_registered(),
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
            sym.is_well_known(),
            "Well-known symbol at index {} should be recognized",
            i
        );
    }

    // Regular symbol should NOT be well-known
    let regular_sym = JsSymbol::new(Some(js_str!("test").into())).unwrap();
    assert!(
        !regular_sym.is_well_known(),
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
        !registered_sym.is_well_known(),
        "Registered symbol should not be well-known"
    );
}
