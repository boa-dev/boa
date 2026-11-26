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
    let mut context = Context::default();
    crate::process::Process::register(&mut context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(TEST_HARNESS),
            TestAction::run(indoc! {r#"
                assert_true(globalThis.hasOwnProperty("process"));  
                assert_equals(typeof process, "object");  
            "#}),
        ],
        &mut context,
    );
}

#[test]
fn process_property_descriptors() {
    let mut context = Context::default();
    crate::process::Process::register(&mut context).unwrap();

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
        &mut context,
    );
}

#[test]
fn process_cwd_method() {
    let mut context = Context::default();
    crate::process::Process::register(&mut context).unwrap();

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
        &mut context,
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
            let mut context = Context::default();
            crate::process::Process::register(&mut context).unwrap();

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
                &mut context,
            );
        },
    );
}

#[test]
#[ignore = "Unsafe under parallel test execution as it tempers with env."]
fn process_env_properties_writable() {
    temp_env::with_var("TEST_VAR", Some("original"), || {
        let mut context = Context::default();
        crate::process::Process::register(&mut context).unwrap();

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
            &mut context,
        );
    });
}

#[test]
fn process_env_object_properties() {
    let mut context = Context::default();
    crate::process::Process::register(&mut context).unwrap();

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
        &mut context,
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
            let mut context = Context::default();
            crate::process::Process::register(&mut context).unwrap();

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
                &mut context,
            );
        },
    );
}
#[test]
fn process_expanded_apis() {
    use crate::process::{Process, ProcessProvider};
    use boa_engine::{Context, Finalize, JsResult, JsString, Source, Trace, js_string};
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    #[derive(Clone, Trace, Finalize)]
    struct MockProvider {
        #[unsafe_ignore_trace]
        cwd: Rc<RefCell<String>>,
        #[unsafe_ignore_trace]
        exit_code: Rc<RefCell<Option<i32>>>,
        #[unsafe_ignore_trace]
        env: Rc<RefCell<HashMap<String, String>>>,
    }

    impl ProcessProvider for MockProvider {
        fn cwd(&self) -> JsResult<JsString> {
            Ok(js_string!(self.cwd.borrow().clone()))
        }
        fn chdir(&self, dir: &JsString) -> JsResult<()> {
            *self.cwd.borrow_mut() = dir.to_std_string_escaped();
            Ok(())
        }
        fn exit(&self, code: i32) -> JsResult<()> {
            *self.exit_code.borrow_mut() = Some(code);
            Ok(())
        }
        fn env(&self) -> impl IntoIterator<Item = (JsString, JsString)> {
            self.env
                .borrow()
                .iter()
                .map(|(k, v)| (js_string!(k.clone()), js_string!(v.clone())))
                .collect::<Vec<_>>()
        }
        fn set_env(&self, key: &JsString, value: &JsString) -> JsResult<()> {
            self.env
                .borrow_mut()
                .insert(key.to_std_string_escaped(), value.to_std_string_escaped());
            Ok(())
        }
        fn remove_env(&self, key: &JsString) -> JsResult<()> {
            self.env.borrow_mut().remove(&key.to_std_string_escaped());
            Ok(())
        }
    }

    let provider = MockProvider {
        cwd: Rc::new(RefCell::new("/initial".to_string())),
        exit_code: Rc::new(RefCell::new(None)),
        env: Rc::new(RefCell::new(HashMap::from([("INIT".into(), "1".into())]))),
    };

    let mut context = Context::default();
    Process::register_with_provider(&mut context, provider.clone()).unwrap();

    let code = r#"
        if (typeof process.platform !== "string" || process.platform.length === 0) throw new Error("platform");
        if (typeof process.arch !== "string" || process.arch.length === 0) throw new Error("arch");
        if (!Array.isArray(process.argv)) throw new Error("argv");
        if (process.cwd() !== "/initial") throw new Error("initial cwd");
        process.chdir("/tmp");
        if (process.cwd() !== "/tmp") throw new Error("updated cwd");
        if (process.env.INIT !== "1") throw new Error("init env");
        process.env.NEW_VAR = "hello";
        if (process.env.NEW_VAR !== "hello") throw new Error("set env");
        delete process.env.NEW_VAR;
        if (process.env.NEW_VAR !== undefined) throw new Error("delete env");
        process.exit(42);
    "#;

    context.eval(Source::from_bytes(code)).unwrap();
    assert_eq!(*provider.exit_code.borrow(), Some(42));
    assert!(!provider.env.borrow().contains_key("NEW_VAR"));
}
