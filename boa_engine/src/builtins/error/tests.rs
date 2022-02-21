use crate::{forward, Context};

#[test]
fn error_to_string() {
    let mut context = Context::default();
    let init = r#"
        let e = new Error('1');
        let name = new Error();
        let message = new Error('message');
        message.name = '';
        let range_e = new RangeError('2');
        let ref_e = new ReferenceError('3');
        let syntax_e = new SyntaxError('4');
        let type_e = new TypeError('5');
    "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "e.toString()"), "\"Error: 1\"");
    assert_eq!(forward(&mut context, "name.toString()"), "\"Error\"");
    assert_eq!(forward(&mut context, "message.toString()"), "\"message\"");
    assert_eq!(
        forward(&mut context, "range_e.toString()"),
        "\"RangeError: 2\""
    );
    assert_eq!(
        forward(&mut context, "ref_e.toString()"),
        "\"ReferenceError: 3\""
    );
    assert_eq!(
        forward(&mut context, "syntax_e.toString()"),
        "\"SyntaxError: 4\""
    );
    assert_eq!(
        forward(&mut context, "type_e.toString()"),
        "\"TypeError: 5\""
    );
}

#[test]
fn eval_error_name() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "EvalError.name"), "\"EvalError\"");
}

#[test]
fn eval_error_length() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "EvalError.length"), "1");
}

#[test]
fn eval_error_to_string() {
    let mut context = Context::default();
    assert_eq!(
        forward(&mut context, "new EvalError('hello').toString()"),
        "\"EvalError: hello\""
    );
    assert_eq!(
        forward(&mut context, "new EvalError().toString()"),
        "\"EvalError\""
    );
}

#[test]
fn uri_error_name() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "URIError.name"), "\"URIError\"");
}

#[test]
fn uri_error_length() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "URIError.length"), "1");
}

#[test]
fn uri_error_to_string() {
    let mut context = Context::default();
    assert_eq!(
        forward(&mut context, "new URIError('hello').toString()"),
        "\"URIError: hello\""
    );
    assert_eq!(
        forward(&mut context, "new URIError().toString()"),
        "\"URIError\""
    );
}
