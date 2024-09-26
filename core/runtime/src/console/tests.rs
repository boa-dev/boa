use super::{formatter, Console, ConsoleState};
use crate::test::{run_test_actions, run_test_actions_with, TestAction};
use crate::{Logger, NullLogger};
use boa_engine::{js_string, property::Attribute, Context, JsError, JsResult, JsValue};
use boa_gc::{Gc, GcRefCell};
use indoc::indoc;

#[test]
fn formatter_no_args_is_empty_string() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(formatter(&[], ctx).unwrap(), "");
    })]);
}

#[test]
fn formatter_empty_format_string_is_empty_string() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(formatter(&[JsValue::new(js_string!())], ctx).unwrap(), "");
    })]);
}

#[test]
fn formatter_format_without_args_renders_verbatim() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(
            formatter(&[JsValue::new(js_string!("%d %s %% %f"))], ctx).unwrap(),
            "%d %s %% %f"
        );
    })]);
}

#[test]
fn formatter_empty_format_string_concatenates_rest_of_args() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(
            formatter(
                &[
                    JsValue::new(js_string!("")),
                    JsValue::new(js_string!("to powinno zostać")),
                    JsValue::new(js_string!("połączone")),
                ],
                ctx
            )
            .unwrap(),
            " to powinno zostać połączone"
        );
    })]);
}

#[test]
fn formatter_utf_8_checks() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(
            formatter(
                &[
                    JsValue::new(js_string!("Są takie chwile %dą %są tu%sów %привет%ź")),
                    JsValue::new(123),
                    JsValue::new(1.23),
                    JsValue::new(js_string!("ł")),
                ],
                ctx
            )
            .unwrap(),
            "Są takie chwile 123ą 1.23ą tułów %привет%ź"
        );
    })]);
}

#[test]
fn formatter_trailing_format_leader_renders() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(
            formatter(
                &[
                    JsValue::new(js_string!("%%%%%")),
                    JsValue::new(js_string!("|"))
                ],
                ctx
            )
            .unwrap(),
            "%%% |"
        );
    })]);
}

#[test]
#[allow(clippy::approx_constant)]
fn formatter_float_format_works() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(
            formatter(&[JsValue::new(js_string!("%f")), JsValue::new(3.1415)], ctx).unwrap(),
            "3.141500"
        );
    })]);
}

#[test]
fn console_log_cyclic() {
    let mut context = Context::default();
    let console = Console::init_with_logger(&mut context, NullLogger);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
                let a = [1];
                a[1] = a;
                console.log(a);
            "#})],
        &mut context,
    );
    // Should not stack overflow
}

/// A logger that records all log messages.
#[derive(Clone, Debug, Default, boa_engine::Trace, boa_engine::Finalize)]
struct RecordingLogger {
    log: Gc<GcRefCell<String>>,
}

impl Logger for RecordingLogger {
    fn log(&self, msg: String, state: &ConsoleState, _: &mut Context) -> JsResult<()> {
        use std::fmt::Write;
        let indent = state.indent();
        writeln!(self.log.borrow_mut(), "{msg:>indent$}").map_err(JsError::from_rust)
    }

    fn info(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    fn warn(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    fn error(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }
}

/// Harness methods to be used in JS tests.
const TEST_HARNESS: &str = r#"
function assert_true(condition, message) {
    if (!condition) {
        throw new Error(`Assertion failed: ${message}`);
    }
}
function assert_own_property(obj, prop) {
    assert_true(
        Object.prototype.hasOwnProperty.call(obj, prop),
        `Expected ${prop.toString()} to be an own property`,
    );
}
function assert_equals(actual, expected, message) {
    assert_true(
        actual === expected,
        `${message} (actual: ${actual.toString()}, expected: ${expected.toString()})`,
    );
}
function assert_throws_js(error, func) {
    try {
        func();
    } catch (e) {
        if (e instanceof error) {
            return;
        }
        throw new Error(`Expected ${error.name} to be thrown, but got ${e.name}`);
    }
    throw new Error(`Expected ${error.name} to be thrown, but no exception was thrown`);
}

// To keep the tests as close to the WPT tests as possible, we define `self` to
// be `globalThis`.
const self = globalThis;
"#;

/// The WPT test `console/console-log-symbol.any.js`.
#[test]
fn wpt_log_symbol_any() {
    let mut context = Context::default();
    let logger = RecordingLogger::default();
    Console::register_with_logger(&mut context, logger.clone()).unwrap();

    run_test_actions_with(
        [
            TestAction::run(TEST_HARNESS),
            TestAction::run(indoc! {r#"
            console.log(Symbol());
            console.log(Symbol("abc"));
            console.log(Symbol.for("def"));
            console.log(Symbol.isConcatSpreadable);
        "#}),
        ],
        &mut context,
    );

    let logs = logger.log.borrow().clone();
    assert_eq!(
        logs,
        indoc! { r#"
            Symbol()
            Symbol(abc)
            Symbol(def)
            Symbol(Symbol.isConcatSpreadable)
        "# }
    );
}

/// The WPT test `console/console-is-a-namespace.any.js`.
#[test]
fn wpt_console_is_a_namespace() {
    let mut context = Context::default();
    let logger = RecordingLogger::default();
    Console::register_with_logger(&mut context, logger.clone()).unwrap();

    run_test_actions_with(
        [
            TestAction::run(TEST_HARNESS),
            // console exists on the global object
            TestAction::run(indoc! {r#"
                assert_true(globalThis.hasOwnProperty("console"));
            "#}),
            // console has the right property descriptors
            TestAction::run(indoc! {r#"
                const propDesc = Object.getOwnPropertyDescriptor(self, "console");
                assert_equals(propDesc.writable, true, "must be writable");
                assert_equals(propDesc.enumerable, false, "must not be enumerable");
                assert_equals(propDesc.configurable, true, "must be configurable");
                assert_equals(propDesc.value, console, "must have the right value");
            "#}),
            // The prototype chain must be correct
            TestAction::run(indoc! {r#"
                const prototype1 = Object.getPrototypeOf(console);
                const prototype2 = Object.getPrototypeOf(prototype1);

                assert_equals(Object.getOwnPropertyNames(prototype1).length, 0, "The [[Prototype]] must have no properties");
                assert_equals(prototype2, Object.prototype, "The [[Prototype]]'s [[Prototype]] must be %ObjectPrototype%");
            "#}),
        ],
        &mut context,
    );
}

/// The WPT test `console/console-label-conversion.any.js`.
#[test]
fn wpt_console_label_conversion() {
    let mut context = Context::default();
    let logger = RecordingLogger::default();
    Console::register_with_logger(&mut context, logger.clone()).unwrap();

    run_test_actions_with(
        [
            TestAction::run(TEST_HARNESS),
            TestAction::run(indoc! {r#"
                const methods = ['count', 'countReset', 'time', 'timeLog', 'timeEnd'];
            "#}),
            // console.${method}()'s label gets converted to string via label.toString() when label is an object
            TestAction::run(indoc! {r#"
                for (const method of methods) {
                    let labelToStringCalled = false;

                    console[method]({
                        toString() {
                            labelToStringCalled = true;
                        }
                    });

                    assert_true(labelToStringCalled, `${method}() must call toString() on label when label is an object`);
                }
            "#}),
            // ${method} must re-throw any exceptions thrown by label.toString() conversion
            TestAction::run(indoc! {r#"
                for (const method of methods) {
                    assert_throws_js(Error, () => {
                        console[method]({
                            toString() {
                                throw new Error('conversion error');
                            }
                        });
                    });
                }
            "#}),
        ],
        &mut context,
    );
}

/// The WPT test `console/console-namespace-object-class-string.any.js`.
#[test]
fn console_namespace_object_class_string() {
    let mut context = Context::default();
    let logger = RecordingLogger::default();
    Console::register_with_logger(&mut context, logger.clone()).unwrap();

    run_test_actions_with(
        [
            TestAction::run(TEST_HARNESS),
            // @@toStringTag exists on the namespace object with the appropriate descriptor
            TestAction::run(indoc! {r#"
                assert_own_property(console, Symbol.toStringTag);

                const propDesc = Object.getOwnPropertyDescriptor(console, Symbol.toStringTag);
                assert_equals(propDesc.value, "console", "value");
                assert_equals(propDesc.writable, false, "writable");
                assert_equals(propDesc.enumerable, false, "enumerable");
                assert_equals(propDesc.configurable, true, "configurable");
            "#}),
            // Object.prototype.toString applied to the namespace object
            TestAction::run(indoc! {r#"
                assert_equals(console.toString(), "[object console]");
                assert_equals(Object.prototype.toString.call(console), "[object console]");
            "#}),
            // Object.prototype.toString applied after modifying the namespace object's @@toStringTag
            TestAction::run(indoc! {r#"
                assert_own_property(console, Symbol.toStringTag, "Precondition: @@toStringTag on the namespace object");
                // t.add_cleanup(() => {
                //   Object.defineProperty(console, Symbol.toStringTag, { value: "console" });
                // });

                Object.defineProperty(console, Symbol.toStringTag, { value: "Test" });
                assert_equals(console.toString(), "[object Test]");
                assert_equals(Object.prototype.toString.call(console), "[object Test]");
            "#}),
            // Object.prototype.toString applied after deleting @@toStringTag
            TestAction::run(indoc! {r#"
                assert_own_property(console, Symbol.toStringTag, "Precondition: @@toStringTag on the namespace object");
                // t.add_cleanup(() => {
                //   Object.defineProperty(console, Symbol.toStringTag, { value: "console" });
                // });

                assert_true(delete console[Symbol.toStringTag]);
                assert_equals(console.toString(), "[object Object]");
                assert_equals(Object.prototype.toString.call(console), "[object Object]");
            "#}),
        ],
        &mut context,
    );
}
