use indoc::indoc;

use crate::{
    builtins::error::ErrorKind,
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject},
    property::{Attribute, PropertyDescriptor},
    run_test_actions, JsValue, TestAction,
};

#[allow(clippy::float_cmp)]
#[test]
fn arguments_object() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function jason(a, b) {
                    return arguments[0];
                }
            "#}),
        TestAction::assert_eq("jason(100, 6)", 100),
    ]);
}

#[test]
fn self_mutating_function_when_calling() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function x() {
                    x.y = 3;
                }
                x();
            "#}),
        TestAction::assert_eq("x.y", 3),
    ]);
}

#[test]
fn self_mutating_function_when_constructing() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function x() {
                    x.y = 3;
                }
                new x();
            "#}),
        TestAction::assert_eq("x.y", 3),
    ]);
}

#[test]
fn function_prototype() {
    run_test_actions([
        TestAction::assert_eq("Function.prototype.name", ""),
        TestAction::assert_eq("Function.prototype.length", 0),
        TestAction::assert_eq("Function.prototype()", JsValue::undefined()),
        TestAction::assert_eq(
            "Function.prototype(1, '', new String(''))",
            JsValue::undefined(),
        ),
        TestAction::assert_native_error(
            "new Function.prototype()",
            ErrorKind::Type,
            "not a constructor",
        ),
    ]);
}

#[test]
fn function_prototype_call() {
    run_test_actions([TestAction::assert_eq(
        "Object.prototype.toString.call(new Error())",
        "[object Error]",
    )]);
}

#[test]
fn function_prototype_call_throw() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            let call = Function.prototype.call;
            call(call)
        "#},
        ErrorKind::Type,
        "undefined is not a function",
    )]);
}

#[test]
fn function_prototype_call_multiple_args() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            function f(a, b) {
                this.a = a;
                this.b = b;
            }
            let o = {a: 0, b: 0};
            f.call(o, 1, 2);
        "#}),
        TestAction::assert_eq("o.a", 1),
        TestAction::assert_eq("o.b", 2),
    ]);
}

#[test]
fn function_prototype_apply() {
    run_test_actions([
        TestAction::run("const numbers = [6, 7, 3, 4, 2]"),
        TestAction::assert_eq("Math.max.apply(null, numbers)", 7),
        TestAction::assert_eq("Math.min.apply(null, numbers)", 2),
    ]);
}

#[test]
fn function_prototype_apply_on_object() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function f(a, b) {
                    this.a = a;
                    this.b = b;
                }
                let o = {a: 0, b: 0};
                f.apply(o, [1, 2]);
            "#}),
        TestAction::assert_eq("o.a", 1),
        TestAction::assert_eq("o.b", 2),
    ]);
}

#[test]
fn closure_capture_clone() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let string = js_string!("Hello");
            let object = JsObject::with_object_proto(ctx);
            object
                .define_property_or_throw(
                    "key",
                    PropertyDescriptor::builder()
                        .value(" world!")
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                    ctx,
                )
                .unwrap();

            let func = FunctionObjectBuilder::new(
                ctx,
                NativeFunction::from_copy_closure_with_captures(
                    |_, _, captures, context| {
                        let (string, object) = &captures;

                        let hw = js_string!(
                            string,
                            &object
                                .__get_own_property__(&"key".into(), context)?
                                .and_then(|prop| prop.value().cloned())
                                .and_then(|val| val.as_string().cloned())
                                .ok_or_else(
                                    || JsNativeError::typ().with_message("invalid `key` property")
                                )?
                        );
                        Ok(hw.into())
                    },
                    (string, object),
                ),
            )
            .name("closure")
            .build();

            ctx.register_global_property("closure", func, Attribute::default());
        }),
        TestAction::assert_eq("closure()", "Hello world!"),
    ]);
}

#[test]
fn function_constructor_early_errors_super() {
    run_test_actions([
        TestAction::assert_native_error(
            "Function('super()')()",
            ErrorKind::Syntax,
            "invalid `super` call",
        ),
        TestAction::assert_native_error(
            "Function('super.a')()",
            ErrorKind::Syntax,
            "invalid `super` reference",
        ),
    ]);
}
