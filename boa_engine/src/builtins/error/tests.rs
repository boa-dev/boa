use crate::{run_test, TestAction};
use indoc::indoc;

#[test]
fn error_to_string() {
    run_test([
        TestAction::assert_eq("(new Error('1')).toString()", "Error: 1"),
        TestAction::assert_eq("(new RangeError('2')).toString()", "RangeError: 2"),
        TestAction::assert_eq("(new ReferenceError('3')).toString()", "ReferenceError: 3"),
        TestAction::assert_eq("(new SyntaxError('4')).toString()", "SyntaxError: 4"),
        TestAction::assert_eq("(new TypeError('5')).toString()", "TypeError: 5"),
        TestAction::assert_eq("(new EvalError('6')).toString()", "EvalError: 6"),
        TestAction::assert_eq("(new URIError('7')).toString()", "URIError: 7"),
        // no message
        TestAction::assert_eq("(new Error()).toString()", "Error"),
        TestAction::assert_eq("(new RangeError()).toString()", "RangeError"),
        TestAction::assert_eq("(new ReferenceError()).toString()", "ReferenceError"),
        TestAction::assert_eq("(new SyntaxError()).toString()", "SyntaxError"),
        TestAction::assert_eq("(new TypeError()).toString()", "TypeError"),
        TestAction::assert_eq("(new EvalError()).toString()", "EvalError"),
        TestAction::assert_eq("(new URIError()).toString()", "URIError"),
        // no name
        TestAction::assert_eq(
            indoc! {r#"
                let message = new Error('message');
                message.name = '';
                message.toString()
            "#},
            "message",
        ),
    ]);
}

#[test]
fn error_names() {
    run_test([
        TestAction::assert_eq("Error.name", "Error"),
        TestAction::assert_eq("EvalError.name", "EvalError"),
        TestAction::assert_eq("RangeError.name", "RangeError"),
        TestAction::assert_eq("ReferenceError.name", "ReferenceError"),
        TestAction::assert_eq("SyntaxError.name", "SyntaxError"),
        TestAction::assert_eq("URIError.name", "URIError"),
        TestAction::assert_eq("TypeError.name", "TypeError"),
        TestAction::assert_eq("AggregateError.name", "AggregateError"),
    ]);
}

#[test]
fn error_lengths() {
    run_test([
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
