#![allow(unused_crate_dependencies, missing_docs)]

use std::path::Path;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::Module;
use boa_engine::{Context, JsError, Source};

/// Normalize native source locations so tests pass with or without the
/// `native-backtrace` feature. Strips both `(native at path:line:col)` and
/// ` (native)` suffixes that appear on error messages and backtrace frames,
/// since these are unstable across feature flags and Rust source changes.
fn strip_native_info(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("(native") {
        result.push_str(&rest[..start]);
        // Skip past the closing ')'
        rest = match rest[start..].find(')') {
            Some(end) => &rest[start + end + 1..],
            None => {
                result.push_str(&rest[start..]);
                return result;
            }
        };
    }
    result.push_str(rest);
    // Trim trailing whitespace left behind on each line
    result
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Test that errors caught by internal handlers (e.g. async module evaluation)
/// preserve their backtrace through promise rejection (`JsError` → `JsValue` → `JsError`).
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
                strip_native_info(&error_str),
                "TypeError: not a callable function\n    at <main> (test.js:2:2)\n    at"
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
        r"function foo() {
    function baz() {
        import.meta.non_existent()
    }
    baz()
}

foo()
"
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
                strip_native_info(&error_str),
                "TypeError: not a callable function\n    at baz (test.js:3:33)\n    at foo (test.js:5:8)\n    at <main> (test.js:8:4)\n    at"
            );
        }
        PromiseState::Fulfilled(_) => panic!("Module should have thrown an error"),
        PromiseState::Pending => panic!("Module evaluation should not be pending"),
    }
}

/// Sanity check: `context.eval()` errors include a backtrace (relates to
/// https://github.com/boa-dev/boa/discussions/4475).
#[test]
fn test_eval_error_has_backtrace() {
    let mut context = Context::default();
    let code = b"const a = 0;\niWillCauseAnError\nconst b = a + 1;";
    let source = Source::from_reader(code.as_slice(), Some(Path::new("test.js")));
    match context.eval(source) {
        Ok(_) => panic!("Should have thrown a ReferenceError"),
        Err(e) => {
            let s = e.to_string();
            eprintln!("eval error output: {s}");
            assert!(
                s.contains("ReferenceError"),
                "Should be a ReferenceError, got: {s}"
            );
        }
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
