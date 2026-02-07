#![allow(unused_crate_dependencies, missing_docs)]

use std::path::Path;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::Module;
use boa_engine::{Context, JsError, Source};

/// Test that errors caught by internal handlers (e.g. async module evaluation)
/// preserve their backtrace through promise rejection (JsError → JsValue → JsError).
#[test]
fn test_call_error_preserves_backtrace() {
    let mut context = Context::default();

    // This module will throw a TypeError at the top level when it tries to
    // call undefined as a function. The error goes through async module
    // evaluation's internal handler → promise rejection → JsError::from_opaque.
    let source = Source::from_bytes(
        b"let x = undefined;
x()",
    )
    .with_path(Path::new("test.js"));

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Rejected(err) => {
            let js_error = JsError::from_opaque(err);
            let error_str = js_error.to_string();
            // The error message should contain backtrace info with "at" entries,
            // proving the backtrace survived the JsError → JsValue → JsError
            // round-trip through promise rejection.
            assert_eq!(
                error_str,
                r#"TypeError: not a callable function
    at <main> (test.js:2:2)
    at  (native)"#
            );
        }
        PromiseState::Fulfilled(_) => panic!("Module should have thrown an error"),
        PromiseState::Pending => panic!("Module evaluation should not be pending"),
    }
}

/// Test that nested call frames produce a full backtrace through the
/// promise rejection round-trip.
#[test]
fn test_nested_call_error_preserves_backtrace() {
    let mut context = Context::default();

    let source = Source::from_bytes(
        r#"function foo() {
    function baz() {
        import.meta.non_existent()
    }
    baz()
}

foo()
"#
        .as_bytes(),
    )
    .with_path(Path::new("test.js"));

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Rejected(err) => {
            let js_error = JsError::from_opaque(err);
            let error_str = js_error.to_string();
            assert_eq!(
                error_str,
                r#"TypeError: not a callable function
    at baz (test.js:3:33)
    at foo (test.js:5:8)
    at <main> (test.js:8:4)
    at  (native)"#
            );
        }
        PromiseState::Fulfilled(_) => panic!("Module should have thrown an error"),
        PromiseState::Pending => panic!("Module evaluation should not be pending"),
    }
}

/// Test that an explicit `throw new Error(...)` inside a module also preserves
/// the backtrace through the promise rejection round-trip.
#[test]
fn test_explicit_throw_preserves_backtrace() {
    let mut context = Context::default();

    let source = Source::from_bytes(
        b"function foo() {
    throw new Error(\"test\")
}
foo()",
    )
    .with_path(Path::new("test.js"));

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Rejected(err) => {
            let js_error = JsError::from_opaque(err);
            let error_str = js_error.to_string();
            // Verify the backtrace contains the expected frames.
            assert!(
                error_str.contains("Error: test"),
                "Should contain error message, got: {error_str}"
            );
            assert!(
                error_str.contains("at foo (test.js:"),
                "Should contain 'foo' frame with file, got: {error_str}"
            );
            assert!(
                error_str.contains("at <main> (test.js:"),
                "Should contain '<main>' frame with file, got: {error_str}"
            );
        }
        PromiseState::Fulfilled(_) => panic!("Module should have thrown an error"),
        PromiseState::Pending => panic!("Module evaluation should not be pending"),
    }
}
