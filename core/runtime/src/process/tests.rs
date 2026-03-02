use crate::test::{TestAction, run_test_actions_with};
use boa_engine::Context;
use indoc::indoc;

/// Test harness functions for process tests.
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
const self = globalThis;
"#;

#[test]
fn process_object_registration() {
    let context = Context::default();
    crate::process::Process::register(&context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(TEST_HARNESS),
            TestAction::run(indoc! {r#"
                assert_true(globalThis.hasOwnProperty("process"));
                assert_equals(typeof process, "object");
            "#}),
        ],
        &context,
    );
}

#[test]
fn process_property_descriptors() {
    let context = Context::default();
    crate::process::Process::register(&context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(TEST_HARNESS),
            TestAction::run(indoc! {r#"
                const propDesc = Object.getOwnPropertyDescriptor(self, "process");
                assert_equals(propDesc.writable, false, "must not be writable");
                assert_equals(propDesc.enumerable, false, "must not be enumerable");
                assert_equals(propDesc.configurable, true, "must be configurable");
                assert_equals(propDesc.value, process, "must have the right value");
            "#}),
        ],
        &context,
    );
}

#[test]
fn process_cwd_method() {
    let context = Context::default();
    crate::process::Process::register(&context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(TEST_HARNESS),
            TestAction::run(indoc! {r#"
                assert_own_property(process, "cwd");
                assert_equals(typeof process.cwd, "function");

                const cwd = process.cwd();
                assert_equals(typeof cwd, "string");
                assert_true(cwd.length > 0, "cwd should not be empty");

                const cwdDesc = Object.getOwnPropertyDescriptor(process, "cwd");
                assert_equals(cwdDesc.writable, true, "cwd must be writable");
                assert_equals(cwdDesc.enumerable, false, "cwd must not be enumerable");
                assert_equals(cwdDesc.configurable, true, "cwd must be configurable");
            "#}),
        ],
        &context,
    );
}

#[test]
#[ignore = "Unsafe under parallel test execution as it tempers with env."]
fn process_env_contains_variables() {
    temp_env::with_vars(
        [
            ("TEST_VAR", Some("test_value")),
            ("ANOTHER_VAR", Some("another_value")),
        ],
        || {
            let context = Context::default();
            crate::process::Process::register(&context).unwrap();

            run_test_actions_with(
                [
                    TestAction::run(TEST_HARNESS),
                    TestAction::run(indoc! {r#"
                    assert_own_property(process, "env");
                    assert_equals(typeof process.env, "object");
                    assert_equals(process.env.TEST_VAR, "test_value");
                    assert_equals(process.env.ANOTHER_VAR, "another_value");
                "#}),
                ],
                &context,
            );
        },
    );
}

#[test]
#[ignore = "Unsafe under parallel test execution as it tempers with env."]
fn process_env_properties_writable() {
    temp_env::with_var("TEST_VAR", Some("original"), || {
        let context = Context::default();
        crate::process::Process::register(&context).unwrap();

        run_test_actions_with(
            [
                TestAction::run(TEST_HARNESS),
                TestAction::run(indoc! {r#"
                    // Test that env properties are writable
                    process.env.TEST_VAR = "modified";
                    assert_equals(process.env.TEST_VAR, "modified");

                    // Test adding new properties
                    process.env.NEW_VAR = "new_value";
                    assert_equals(process.env.NEW_VAR, "new_value");
                "#}),
            ],
            &context,
        );
    });
}

#[test]
fn process_env_object_properties() {
    let context = Context::default();
    crate::process::Process::register(&context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(TEST_HARNESS),
            TestAction::run(indoc! {r#"
                const envDesc = Object.getOwnPropertyDescriptor(process, "env");
                assert_equals(envDesc.writable, true, "env must be writable");
                assert_equals(envDesc.enumerable, false, "env must not be enumerable");
                assert_equals(envDesc.configurable, true, "env must be configurable");
                assert_equals(typeof envDesc.value, "object", "env must be an object");
            "#}),
        ],
        &context,
    );
}

#[test]
#[ignore = "Unsafe under parallel test execution as it tempers with env."]
fn process_env_iteration() {
    temp_env::with_vars(
        [
            ("ITER_TEST_1", Some("value1")),
            ("ITER_TEST_2", Some("value2")),
        ],
        || {
            let context = Context::default();
            crate::process::Process::register(&context).unwrap();

            run_test_actions_with(
                [
                    TestAction::run(TEST_HARNESS),
                    TestAction::run(indoc! {r#"
                    let found1 = false, found2 = false;
                    for (let key in process.env) {
                        if (key === "ITER_TEST_1") {
                            assert_equals(process.env[key], "value1");
                            found1 = true;
                        }
                        if (key === "ITER_TEST_2") {
                            assert_equals(process.env[key], "value2");
                            found2 = true;
                        }
                    }
                    assert_true(found1, "ITER_TEST_1 should be found");
                    assert_true(found2, "ITER_TEST_2 should be found");
                "#}),
                ],
                &context,
            );
        },
    );
}
