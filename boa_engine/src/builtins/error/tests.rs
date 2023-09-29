use crate::{js_string, run_test_actions, TestAction};
use indoc::indoc;

#[test]
fn error_to_string() {
    run_test_actions([
        TestAction::assert_eq("(new Error('1')).toString()", js_string!("Error: 1")),
        TestAction::assert_eq(
            "(new RangeError('2')).toString()",
            js_string!("RangeError: 2"),
        ),
        TestAction::assert_eq(
            "(new ReferenceError('3')).toString()",
            js_string!("ReferenceError: 3"),
        ),
        TestAction::assert_eq(
            "(new SyntaxError('4')).toString()",
            js_string!("SyntaxError: 4"),
        ),
        TestAction::assert_eq(
            "(new TypeError('5')).toString()",
            js_string!("TypeError: 5"),
        ),
        TestAction::assert_eq(
            "(new EvalError('6')).toString()",
            js_string!("EvalError: 6"),
        ),
        TestAction::assert_eq("(new URIError('7')).toString()", js_string!("URIError: 7")),
        // no message
        TestAction::assert_eq("(new Error()).toString()", js_string!("Error")),
        TestAction::assert_eq("(new RangeError()).toString()", js_string!("RangeError")),
        TestAction::assert_eq(
            "(new ReferenceError()).toString()",
            js_string!("ReferenceError"),
        ),
        TestAction::assert_eq("(new SyntaxError()).toString()", js_string!("SyntaxError")),
        TestAction::assert_eq("(new TypeError()).toString()", js_string!("TypeError")),
        TestAction::assert_eq("(new EvalError()).toString()", js_string!("EvalError")),
        TestAction::assert_eq("(new URIError()).toString()", js_string!("URIError")),
        // no name
        TestAction::assert_eq(
            indoc! {r#"
                let message = new Error('message');
                message.name = '';
                message.toString()
            "#},
            js_string!("message"),
        ),
    ]);
}

#[test]
fn error_names() {
    run_test_actions([
        TestAction::assert_eq("Error.name", js_string!("Error")),
        TestAction::assert_eq("EvalError.name", js_string!("EvalError")),
        TestAction::assert_eq("RangeError.name", js_string!("RangeError")),
        TestAction::assert_eq("ReferenceError.name", js_string!("ReferenceError")),
        TestAction::assert_eq("SyntaxError.name", js_string!("SyntaxError")),
        TestAction::assert_eq("URIError.name", js_string!("URIError")),
        TestAction::assert_eq("TypeError.name", js_string!("TypeError")),
        TestAction::assert_eq("AggregateError.name", js_string!("AggregateError")),
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
