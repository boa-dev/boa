use crate::{run_test_actions, TestAction};
use boa_macros::js_str;
use indoc::indoc;

#[test]
fn class_field_initializer_name_static() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class C {
                static a = function() {};
                static ["b"] = function() {};
                static #c = function() {};
                static c = this.#c
            }
        "#}),
        TestAction::assert_eq("C.a.name", js_str!("a")),
        TestAction::assert_eq("C.b.name", js_str!("b")),
        TestAction::assert_eq("C.c.name", js_str!("#c")),
    ]);
}

#[test]
fn class_field_initializer_name() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class C {
                a = function() {};
                ["b"] = function() {};
                #c = function() {};
                c = this.#c
            }
            let c = new C();
        "#}),
        TestAction::assert_eq("c.a.name", js_str!("a")),
        TestAction::assert_eq("c.b.name", js_str!("b")),
        TestAction::assert_eq("c.c.name", js_str!("#c")),
    ]);
}

#[test]
fn class_superclass_from_regex_error() {
    run_test_actions([TestAction::assert_native_error(
        "class A extends /=/ {}",
        crate::JsNativeErrorKind::Type,
        "superclass must be a constructor",
    )]);
}
