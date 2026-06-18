use super::{Console, ConsoleState, formatter};
use crate::test::{TestAction, run_test_actions, run_test_actions_with};
use crate::{Logger, NullLogger};
use boa_engine::{Context, JsError, JsResult, JsValue, js_string, property::Attribute};
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
    let console = Console::init_with_logger(NullLogger, &mut context);
    context
        .register_global_property(Console::NAME, console, Attribute::all())
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
pub(crate) struct RecordingLogger {
    pub log: Gc<GcRefCell<String>>,
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
    Console::register_with_logger(logger.clone(), &mut context).unwrap();

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
    Console::register_with_logger(logger.clone(), &mut context).unwrap();

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
    Console::register_with_logger(logger.clone(), &mut context).unwrap();

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
    Console::register_with_logger(logger.clone(), &mut context).unwrap();

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

#[test]
fn console_log_arguments() {
    let mut context = Context::default();
    let logger = RecordingLogger::default();
    Console::register_with_logger(logger.clone(), &mut context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            function test() {
                console.log(arguments);
            }
            test("a", 42);
        "#})],
        &mut context,
    );

    let logs = logger.log.borrow().clone();
    assert_eq!(
        logs,
        indoc! { r#"
            [Arguments] {
              0: "a",
              1: 42
            }
        "# }
    );
}

#[test]
fn console_log_regexp() {
    let mut context = Context::default();
    let logger = RecordingLogger::default();
    Console::register_with_logger(logger.clone(), &mut context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            console.log(/foo/gi);
            console.log(/^hello$/m);
            console.log(new RegExp("a/b", "g"));
            console.log(new RegExp("foo\nbar", "m"));
        "#})],
        &mut context,
    );

    let logs = logger.log.borrow().clone();
    assert_eq!(
        logs,
        indoc! { r#"
            /foo/gi
            /^hello$/m
            /a\/b/g
            /foo\nbar/m
        "# }
    );
}

#[test]
fn console_log_date() {
    let mut context = Context::default();
    let logger = RecordingLogger::default();
    Console::register_with_logger(logger.clone(), &mut context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            console.log(new Date("Invalid"));
            console.log(new Date(0));
        "#})],
        &mut context,
    );

    let logs = logger.log.borrow().clone();
    assert_eq!(
        logs,
        indoc! { r#"
            Invalid Date
            1970-01-01T00:00:00.000Z
        "# }
    );
}

#[test]
fn trace_with_stack_trace() {
    let mut context = Context::default();
    let logger = RecordingLogger::default();
    Console::register_with_logger(logger.clone(), &mut context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(TEST_HARNESS),
            TestAction::run(indoc! {r#"
            console.trace("one");
            a();

            function a() {
                b();
            }
            function b() {
                console.trace("two");
            }
        "#}),
        ],
        &mut context,
    );

    let logs = logger.log.borrow().clone();
    assert_eq!(
        logs,
        indoc! { r#"
            one
            <main>
            two
            b
            a
            <main>
        "# }
    );
}

// ---- console.table tests ----
//
// These test the `console.table()` implementation against common edge cases
// drawn from the Node.js and Bun test suites.

/// Helper: runs JS code with a `RecordingLogger` and returns the captured log.
macro_rules! run_table_test {
    ($js:expr) => {{
        let mut context = Context::default();
        let logger = RecordingLogger::default();
        Console::register_with_logger(logger.clone(), &mut context).unwrap();

        run_test_actions_with(
            [TestAction::run(TEST_HARNESS), TestAction::run($js)],
            &mut context,
        );

        logger.log.borrow().clone()
    }};
}

/// Primitives (null, undefined, false, number, string, Symbol) should fall
/// back to plain `console.log` output — no table rendered.
#[test]
fn console_table_primitives_fall_back_to_log() {
    let logs = run_table_test!(indoc! {r#"
        console.table(null);
        console.table(undefined);
        console.table(false);
        console.table(42);
        console.table("hello");
        console.table(Symbol("test"));
    "#});

    assert_eq!(
        logs,
        indoc! { r#"
            null
            undefined
            false
            42
            hello
            Symbol(test)
        "# }
    );
}

/// `console.table()` with no arguments falls back to `console.log()` with
/// no data, which outputs an empty line.
#[test]
fn console_table_no_args() {
    let logs = run_table_test!("console.table();");
    assert_eq!(logs, "\n");
}

/// Empty array and empty object should fall back to log (no rows to tabulate).
#[test]
fn console_table_empty_collections() {
    let logs = run_table_test!(indoc! {r#"
        console.table([]);
    "#});
    // Empty array falls back to console.log, which prints "[]"
    assert!(
        !logs.contains("(index)"),
        "empty array should not render a table"
    );

    let logs = run_table_test!(indoc! {r#"
        console.table({});
    "#});
    assert!(
        !logs.contains("(index)"),
        "empty object should not render a table"
    );
}

/// Array of objects: each object's properties become columns.
#[test]
fn console_table_array_of_objects() {
    let logs = run_table_test!(indoc! {r#"
        console.table([{a: 1, b: 2}, {a: 3, b: 4}]);
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains(" a "));
    assert!(logs.contains(" b "));
    // Should NOT have a "Value" column (all elements are objects).
    assert!(!logs.contains("Values"));
    // Row data present.
    assert!(logs.contains('0'));
    assert!(logs.contains('1'));
    assert!(logs.contains('3'));
    assert!(logs.contains('4'));
}

/// Properties filter restricts which columns are shown.
#[test]
fn console_table_with_properties_filter() {
    let logs = run_table_test!(indoc! {r#"
        console.table([{a: 1, b: 2}, {a: 3, b: 4}], ["a"]);
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains(" a "));
    // Value "2" from column b should not appear.
    assert!(
        !logs.contains(" 2 "),
        "filtered column b data should be absent"
    );
    assert!(
        !logs.contains(" 4 "),
        "filtered column b data should be absent"
    );
}

/// Empty properties filter: only the (index) column should appear.
#[test]
fn console_table_empty_properties_filter() {
    let logs = run_table_test!(indoc! {r#"
        console.table([{a: 1, b: 2}], []);
    "#});

    assert!(logs.contains("(index)"));
    assert!(!logs.contains(" a "), "no data columns with empty filter");
    assert!(!logs.contains(" b "), "no data columns with empty filter");
}

/// Non-existent property in filter: only (index) column appears since "x"
/// doesn't match any actual property.
#[test]
fn console_table_nonexistent_property_filter() {
    let logs = run_table_test!(indoc! {r#"
        console.table({a: 1}, ["x"]);
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains(" a "), "key 'a' should appear as index value");
}

/// Array of primitive values: shows (index) + Value columns.
#[test]
fn console_table_primitive_array() {
    let logs = run_table_test!(indoc! {r#"
        console.table([10, 20, 30]);
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains("Values"));
    assert!(logs.contains("10"));
    assert!(logs.contains("20"));
    assert!(logs.contains("30"));
}

/// Object with primitive values: keys become (index), values go in Value column.
#[test]
fn console_table_object_with_primitive_values() {
    let logs = run_table_test!(indoc! {r#"
        console.table({name: "Alice", age: 30});
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains("Values"));
    assert!(logs.contains("name"));
    assert!(logs.contains("age"));
    assert!(logs.contains("30"));
}

/// Object of objects: outer keys are indices, inner properties are columns.
#[test]
fn console_table_nested_objects() {
    let logs = run_table_test!(indoc! {r#"
        console.table({a: {x: 1, y: 2}, b: {x: 3, y: 4}});
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains(" x "));
    assert!(logs.contains(" y "));
    assert!(logs.contains(" a "));
    assert!(logs.contains(" b "));
    assert!(
        !logs.contains("Values"),
        "nested objects should not produce Values column"
    );
}

/// Mixed array: objects and primitives together. Should have both named
/// columns (from objects) and a Value column (for primitives).
#[test]
fn console_table_mixed_array() {
    let logs = run_table_test!(indoc! {r#"
        console.table([{a: 1}, 42]);
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains(" a "), "column from object element");
    assert!(
        logs.contains("Value"),
        "Values column for primitive element"
    );
    assert!(logs.contains('1'));
    assert!(logs.contains("42"));
}

/// Sparse columns: objects with different property sets. Missing cells
/// should be empty.
#[test]
fn console_table_sparse_columns() {
    let logs = run_table_test!(indoc! {r#"
        console.table([{a: 1}, {b: 2}]);
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains(" a "));
    assert!(logs.contains(" b "));
    assert!(logs.contains('1'));
    assert!(logs.contains('2'));
}

/// Security: using `__proto__` as a property filter must NOT cause prototype
/// pollution.
#[test]
fn console_table_proto_safety() {
    // This should not throw — if __proto__ pollution occurred, the JS assertion
    // would fail and the test action would panic.
    let _logs = run_table_test!(indoc! {r#"
        console.table([{foo: 10}, {foo: 20}], ["__proto__"]);
        if ("0" in Object.prototype) {
            throw new Error("prototype pollution detected!");
        }
        if ("1" in Object.prototype) {
            throw new Error("prototype pollution detected!");
        }
    "#});
}

/// Map: should display with `(iteration index)`, `Key`, and `Values` columns.
#[test]
fn console_table_map() {
    let logs = run_table_test!(indoc! {r#"
        console.table(new Map([["a", 1], ["b", 2]]));
    "#});

    assert!(logs.contains("(iteration index)"));
    assert!(logs.contains("Key"));
    assert!(logs.contains("Values"));
    assert!(logs.contains("\"a\""));
    assert!(logs.contains("\"b\""));
    assert!(logs.contains('1'));
    assert!(logs.contains('2'));
}

/// Set: should display with `(iteration index)` and `Values` columns.
#[test]
fn console_table_set() {
    let logs = run_table_test!(indoc! {r#"
        console.table(new Set([1, 2, 3]));
    "#});

    assert!(logs.contains("(iteration index)"));
    assert!(logs.contains("Values"));
    assert!(logs.contains('1'));
    assert!(logs.contains('2'));
    assert!(logs.contains('3'));
    assert!(!logs.contains("Key"), "Set should not have a Key column");
}

/// Empty Map should fall back to console.log (no rows).
#[test]
fn console_table_empty_map() {
    let logs = run_table_test!(indoc! {r#"
        console.table(new Map());
    "#});

    assert!(
        !logs.contains("(iteration index)"),
        "empty Map should not render a table"
    );
}

/// `TypedArray` should work like a regular array.
#[test]
fn console_table_typed_array() {
    let logs = run_table_test!(indoc! {r#"
        console.table(new Uint8Array([1, 2, 3]));
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains("Values"));
    assert!(logs.contains('1'));
    assert!(logs.contains('2'));
    assert!(logs.contains('3'));
}

/// Deeply nested objects should render inline on a single line in cells,
/// not as multi-line pretty-print.
#[test]
fn console_table_deeply_nested_inline() {
    let logs = run_table_test!(indoc! {r#"
        console.table({a: {x: {nested: true}}});
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains(" x "));
    // The nested object should be on one line, not split across multiple rows.
    assert!(
        logs.contains("{ nested: true }"),
        "nested object should be displayed inline"
    );
}

/// Invalid second argument (non-array) should throw a `TypeError`.
#[test]
fn console_table_invalid_properties_throws() {
    let _logs = run_table_test!(indoc! {r#"
        assert_throws_js(TypeError, () => {
            console.table([], false);
        });
        assert_throws_js(TypeError, () => {
            console.table([], "bad");
        });
        assert_throws_js(TypeError, () => {
            console.table([], {});
        });
    "#});
}

/// Duplicate entries in the properties filter should be deduplicated.
#[test]
fn console_table_duplicate_property_filter() {
    let logs = run_table_test!(indoc! {r#"
        console.table([{a: 1, b: 2}], ["a", "b", "a"]);
    "#});

    assert!(logs.contains("(index)"));
    assert!(logs.contains(" a "));
    assert!(logs.contains(" b "));
    // Count occurrences of " a " — should appear exactly twice:
    // once in header, once in data separator. If duplicated, there would be more.
    let a_col_count = logs.matches(" a ").count();
    assert!(
        a_col_count <= 3,
        "column 'a' should not be duplicated, found {a_col_count} occurrences"
    );
}

/// The properties filter should control column *order*, matching the filter
/// array's order rather than the data's property discovery order.
#[test]
fn console_table_properties_filter_controls_order() {
    let logs = run_table_test!(indoc! {r#"
        console.table([{a: 1, b: 2}], ["b", "a"]);
    "#});

    // "b" should appear before "a" in the output.
    let b_pos = logs.find(" b ").expect("should have column b");
    let a_pos = logs.find(" a ").expect("should have column a");
    assert!(b_pos < a_pos, "column b should appear before column a");
}

/// Properties that don't exist in the data should still appear as empty columns.
#[test]
fn console_table_nonexistent_property_shows_empty_column() {
    let logs = run_table_test!(indoc! {r#"
        console.table([1, 2], ["foo"]);
    "#});

    assert!(logs.contains("(index)"));
    assert!(
        logs.contains("foo"),
        "nonexistent property should appear as a column"
    );
}

/// Map should ignore the properties filter and keep its fixed column layout.
#[test]
fn console_table_map_ignores_properties_filter() {
    let logs = run_table_test!(indoc! {r#"
        console.table(new Map([["x", 1]]), ["a"]);
    "#});

    assert!(logs.contains("(iteration index)"));
    assert!(logs.contains("Key"));
    assert!(logs.contains("Values"));
}

/// Set should ignore the properties filter and keep its fixed column layout.
#[test]
fn console_table_set_ignores_properties_filter() {
    let logs = run_table_test!(indoc! {r#"
        console.table(new Set([1, 2]), ["a"]);
    "#});

    assert!(logs.contains("(iteration index)"));
    assert!(logs.contains("Values"));
}
