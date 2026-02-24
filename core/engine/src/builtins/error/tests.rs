use std::path::Path;

use crate::{
    Context, JsError, Source, TestAction,
    builtins::promise::PromiseState,
    module::Module,
    run_test_actions,
    vm::{shadow_stack::ShadowEntry, source_info::SourcePath},
};
use boa_macros::js_str;
use indoc::indoc;

/// Helper to extract backtrace entries from a rejected module promise.
fn get_backtrace_from_rejection(
    context: &mut Context,
    js_code: &[u8],
    path: &str,
) -> Vec<ShadowEntry> {
    let source = Source::from_bytes(js_code).with_path(Path::new(path));
    let module = Module::parse(source, None, context).unwrap();
    let promise = module.load_link_evaluate(context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Rejected(err) => {
            let js_error = JsError::from_opaque(err);
            js_error
                .backtrace
                .as_ref()
                .expect("error should have a backtrace")
                .iter()
                .cloned()
                .collect()
        }
        PromiseState::Fulfilled(_) => panic!("Module should have thrown an error"),
        PromiseState::Pending => panic!("Module evaluation should not be pending"),
    }
}

/// Assert that a `ShadowEntry::Bytecode` frame matches expected function name, path, line, and column.
#[track_caller]
fn assert_bytecode_frame(
    entry: &ShadowEntry,
    expected_fn: &str,
    expected_path: &Path,
    expected_line: u32,
    expected_col: u32,
) {
    match entry {
        ShadowEntry::Bytecode { pc, source_info } => {
            assert_eq!(
                source_info.function_name().to_std_string_escaped(),
                expected_fn,
                "function name mismatch"
            );
            assert_eq!(
                source_info.map().path(),
                &SourcePath::Path(expected_path.into()),
                "path mismatch"
            );
            let pos = source_info
                .map()
                .find(*pc)
                .expect("should have a source position");
            assert_eq!(pos.line_number(), expected_line, "line number mismatch");
            assert_eq!(pos.column_number(), expected_col, "column number mismatch");
        }
        ShadowEntry::Native { .. } => panic!("expected Bytecode frame, got Native"),
    }
}

/// Assert that a `ShadowEntry` is a `Native` frame.
#[track_caller]
fn assert_native_frame(entry: &ShadowEntry) {
    assert!(
        matches!(entry, ShadowEntry::Native { .. }),
        "expected Native frame, got Bytecode"
    );
}

/// Test that errors caught by internal handlers (e.g. async module evaluation)
/// preserve their backtrace through promise rejection (`JsError` -> `JsValue` -> `JsError`).
#[test]
fn backtrace_preserved_through_promise_rejection() {
    let mut context = Context::default();
    let entries = get_backtrace_from_rejection(&mut context, b"let x = undefined;\nx()", "test.js");

    let path = Path::new("test.js");

    // Backtrace stored bottom-up: [Native (call site), Bytecode (<main>)]
    assert_eq!(entries.len(), 2, "expected 2 backtrace entries");
    assert_native_frame(&entries[0]);
    assert_bytecode_frame(&entries[1], "<main>", path, 2, 2);
}

/// Test that nested call frames produce a full backtrace through the
/// promise rejection round-trip.
#[test]
fn nested_backtrace_preserved_through_promise_rejection() {
    let mut context = Context::default();
    let entries = get_backtrace_from_rejection(
        &mut context,
        br"function foo() {
    function baz() {
        import.meta.non_existent()
    }
    baz()
}

foo()
",
        "test.js",
    );

    let path = Path::new("test.js");

    // Backtrace stored bottom-up: [Native, <main>, foo, baz]
    assert_eq!(entries.len(), 4, "expected 4 backtrace entries");
    assert_native_frame(&entries[0]);
    assert_bytecode_frame(&entries[1], "<main>", path, 8, 4);
    assert_bytecode_frame(&entries[2], "foo", path, 5, 8);
    assert_bytecode_frame(&entries[3], "baz", path, 3, 33);
}

/// Test that an explicit `throw new Error(...)` inside a module also preserves
/// the backtrace through the promise rejection round-trip.
#[test]
fn explicit_throw_backtrace_preserved_through_promise_rejection() {
    let mut context = Context::default();
    let entries = get_backtrace_from_rejection(
        &mut context,
        b"function foo() {\n    throw new Error(\"test\")\n}\nfoo()",
        "test.js",
    );

    let path = Path::new("test.js");

    // Backtrace stored bottom-up: [Native, <main>, foo]
    assert_eq!(entries.len(), 3, "expected 3 backtrace entries");
    assert_native_frame(&entries[0]);
    assert_bytecode_frame(&entries[1], "<main>", path, 4, 4);
    assert_bytecode_frame(&entries[2], "foo", path, 2, 11);
}

/// Sanity check: `context.eval()` errors include a backtrace (relates to
/// <https://github.com/boa-dev/boa/discussions/4475>).
#[test]
fn eval_error_has_backtrace() {
    let mut context = Context::default();
    let code = b"const a = 0;\niWillCauseAnError\nconst b = a + 1;";
    let source = Source::from_reader(code.as_slice(), Some(Path::new("test.js")));
    match context.eval(source) {
        Ok(_) => panic!("Should have thrown a ReferenceError"),
        Err(e) => {
            assert!(e.backtrace.is_some(), "eval error should have a backtrace");
            let entries: Vec<_> = e.backtrace.as_ref().unwrap().iter().collect();
            assert!(
                !entries.is_empty(),
                "backtrace should have at least one entry"
            );
        }
    }
}

#[test]
fn error_to_string() {
    run_test_actions([
        TestAction::assert_eq("(new Error('1')).toString()", js_str!("Error: 1")),
        TestAction::assert_eq("(new RangeError('2')).toString()", js_str!("RangeError: 2")),
        TestAction::assert_eq(
            "(new ReferenceError('3')).toString()",
            js_str!("ReferenceError: 3"),
        ),
        TestAction::assert_eq(
            "(new SyntaxError('4')).toString()",
            js_str!("SyntaxError: 4"),
        ),
        TestAction::assert_eq("(new TypeError('5')).toString()", js_str!("TypeError: 5")),
        TestAction::assert_eq("(new EvalError('6')).toString()", js_str!("EvalError: 6")),
        TestAction::assert_eq("(new URIError('7')).toString()", js_str!("URIError: 7")),
        // no message
        TestAction::assert_eq("(new Error()).toString()", js_str!("Error")),
        TestAction::assert_eq("(new RangeError()).toString()", js_str!("RangeError")),
        TestAction::assert_eq(
            "(new ReferenceError()).toString()",
            js_str!("ReferenceError"),
        ),
        TestAction::assert_eq("(new SyntaxError()).toString()", js_str!("SyntaxError")),
        TestAction::assert_eq("(new TypeError()).toString()", js_str!("TypeError")),
        TestAction::assert_eq("(new EvalError()).toString()", js_str!("EvalError")),
        TestAction::assert_eq("(new URIError()).toString()", js_str!("URIError")),
        // no name
        TestAction::assert_eq(
            indoc! {r#"
                let message = new Error('message');
                message.name = '';
                message.toString()
            "#},
            js_str!("message"),
        ),
    ]);
}

#[test]
fn error_names() {
    run_test_actions([
        TestAction::assert_eq("Error.name", js_str!("Error")),
        TestAction::assert_eq("EvalError.name", js_str!("EvalError")),
        TestAction::assert_eq("RangeError.name", js_str!("RangeError")),
        TestAction::assert_eq("ReferenceError.name", js_str!("ReferenceError")),
        TestAction::assert_eq("SyntaxError.name", js_str!("SyntaxError")),
        TestAction::assert_eq("URIError.name", js_str!("URIError")),
        TestAction::assert_eq("TypeError.name", js_str!("TypeError")),
        TestAction::assert_eq("AggregateError.name", js_str!("AggregateError")),
    ]);
}

#[test]
fn error_lengths() {
    run_test_actions([
        TestAction::assert_eq("Error.length", 1),
        TestAction::assert_eq("EvalError.length", 1),
        TestAction::assert_eq("RangeError.length", 1),
        TestAction::assert_eq("ReferenceError.length", 1),
        TestAction::assert_eq("SyntaxError.length", 1),
        TestAction::assert_eq("URIError.length", 1),
        TestAction::assert_eq("TypeError.length", 1),
        TestAction::assert_eq("AggregateError.length", 2),
    ]);
}
