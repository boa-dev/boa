use crate::{TestAction, run_test_actions};
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

// https://github.com/boa-dev/boa/issues/3055
#[test]
fn class_can_access_super_from_static_initializer() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class a {
                static field = "super field";
            }

            class b extends a {
                static #field = super.field;
                static get field() {
                    return this.#field;
                }
            }

            class c extends a {
                static field = super.field;
            }

        "#}),
        TestAction::assert_eq("a.field", js_str!("super field")),
        TestAction::assert_eq("b.field", js_str!("super field")),
        TestAction::assert_eq("c.field", js_str!("super field")),
    ]);
}

// https://github.com/boa-dev/boa/issues/4400
#[test]
fn class_in_constructor() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class C {
                constructor() {
                    class D {}
                    this.v = D.name.toString()
                }
            }
            let c = new C()

        "#}),
        TestAction::assert_eq("c.v", js_str!("D")),
    ]);
}
