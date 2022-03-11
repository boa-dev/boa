use crate::{exec, Context, JsValue};

#[test]
fn typeof_string() {
    let typeof_object = r#"
        const a = "hello";
        typeof a;
    "#;
    assert_eq!(&exec(typeof_object), "\"string\"");
}

#[test]
fn typeof_number() {
    let typeof_number = r#"
        let a = 1234;
        typeof a;
    "#;
    assert_eq!(&exec(typeof_number), "\"number\"");
}

#[test]
fn basic_op() {
    let basic_op = r#"
        const a = 1;
        const b = 2;
        a + b
    "#;
    assert_eq!(&exec(basic_op), "3");
}

#[test]
fn try_catch_finally_from_init() {
    // the initialisation of the array here emits a PopOnReturnAdd op
    //
    // here we test that the stack is not popped more than intended due to multiple catches in the
    // same function, which could lead to VM stack corruption
    let source = r#"
        try {
            [(() => {throw "h";})()];
        } catch (x) {
            throw "h";
        } finally {
        }
    "#;

    assert_eq!(Context::default().eval(source.as_bytes()), Err("h".into()));
}

#[test]
fn multiple_catches() {
    // see explanation on `try_catch_finally_from_init`
    let source = r#"
        try {
            try {
                [(() => {throw "h";})()];
            } catch (x) {
                throw "h";
            }
        } catch (y) {
        }
    "#;

    assert_eq!(
        Context::default().eval(source.as_bytes()),
        Ok(JsValue::Undefined)
    );
}
