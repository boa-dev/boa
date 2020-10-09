use crate::{forward, Context};

#[test]
fn error_to_string() {
    let mut ctx = Context::new();
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
    forward(&mut ctx, init);
    assert_eq!(forward(&mut ctx, "e.toString()"), "\"Error: 1\"");
    assert_eq!(forward(&mut ctx, "name.toString()"), "\"Error\"");
    assert_eq!(forward(&mut ctx, "message.toString()"), "\"message\"");
    assert_eq!(forward(&mut ctx, "range_e.toString()"), "\"RangeError: 2\"");
    assert_eq!(
        forward(&mut ctx, "ref_e.toString()"),
        "\"ReferenceError: 3\""
    );
    assert_eq!(
        forward(&mut ctx, "syntax_e.toString()"),
        "\"SyntaxError: 4\""
    );
    assert_eq!(forward(&mut ctx, "type_e.toString()"), "\"TypeError: 5\"");
}

#[test]
fn eval_error_name() {
    let mut ctx = Context::new();
    assert_eq!(forward(&mut ctx, "EvalError.name"), "\"EvalError\"");
}

#[test]
fn eval_error_length() {
    let mut ctx = Context::new();
    assert_eq!(forward(&mut ctx, "EvalError.length"), "1");
}

#[test]
fn eval_error_to_string() {
    let mut ctx = Context::new();
    assert_eq!(
        forward(&mut ctx, "new EvalError('hello').toString()"),
        "\"EvalError: hello\""
    );
    assert_eq!(
        forward(&mut ctx, "new EvalError().toString()"),
        "\"EvalError\""
    );
}

#[test]
fn uri_error_name() {
    let mut ctx = Context::new();
    assert_eq!(forward(&mut ctx, "URIError.name"), "\"URIError\"");
}

#[test]
fn uri_error_length() {
    let mut ctx = Context::new();
    assert_eq!(forward(&mut ctx, "URIError.length"), "1");
}

#[test]
fn uri_error_to_string() {
    let mut ctx = Context::new();
    assert_eq!(
        forward(&mut ctx, "new URIError('hello').toString()"),
        "\"URIError: hello\""
    );
    assert_eq!(
        forward(&mut ctx, "new URIError().toString()"),
        "\"URIError\""
    );
}
