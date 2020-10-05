use crate::{forward, forward_val, Context};

#[allow(clippy::float_cmp)]
#[test]
fn arguments_object() {
    let mut engine = Context::new();

    let init = r#"
        function jason(a, b) {
            return arguments[0];
        }
        var val = jason(100, 6);
        "#;

    eprintln!("{}", forward(&mut engine, init));

    let return_val = forward_val(&mut engine, "val").expect("value expected");
    assert_eq!(return_val.is_integer(), true);
    assert_eq!(
        return_val
            .to_i32(&mut engine)
            .expect("Could not convert value to i32"),
        100
    );
}

#[test]
fn self_mutating_function_when_calling() {
    let mut engine = Context::new();
    let func = r#"
        function x() {
	        x.y = 3;
        }
        x();
        "#;
    eprintln!("{}", forward(&mut engine, func));
    let y = forward_val(&mut engine, "x.y").expect("value expected");
    assert_eq!(y.is_integer(), true);
    assert_eq!(
        y.to_i32(&mut engine)
            .expect("Could not convert value to i32"),
        3
    );
}

#[test]
fn self_mutating_function_when_constructing() {
    let mut engine = Context::new();
    let func = r#"
        function x() {
            x.y = 3;
        }
        new x();
        "#;
    eprintln!("{}", forward(&mut engine, func));
    let y = forward_val(&mut engine, "x.y").expect("value expected");
    assert_eq!(y.is_integer(), true);
    assert_eq!(
        y.to_i32(&mut engine)
            .expect("Could not convert value to i32"),
        3
    );
}

#[test]
fn call_function_prototype() {
    let mut engine = Context::new();
    let func = r#"
        Function.prototype()
        "#;
    let value = forward_val(&mut engine, func).unwrap();
    assert!(value.is_undefined());
}

#[test]
fn call_function_prototype_with_arguments() {
    let mut engine = Context::new();
    let func = r#"
        Function.prototype(1, "", new String(""))
        "#;
    let value = forward_val(&mut engine, func).unwrap();
    assert!(value.is_undefined());
}

#[test]
fn call_function_prototype_with_new() {
    let mut engine = Context::new();
    let func = r#"
        new Function.prototype()
        "#;
    let value = forward_val(&mut engine, func);
    assert!(value.is_err());
}

#[test]
fn function_prototype_name() {
    let mut engine = Context::new();
    let func = r#"
        Function.prototype.name
        "#;
    let value = forward_val(&mut engine, func).unwrap();
    assert!(value.is_string());
    assert!(value.as_string().unwrap().is_empty());
}

#[test]
#[allow(clippy::float_cmp)]
fn function_prototype_length() {
    let mut engine = Context::new();
    let func = r#"
        Function.prototype.length
        "#;
    let value = forward_val(&mut engine, func).unwrap();
    assert!(value.is_number());
    assert_eq!(value.as_number().unwrap(), 0.0);
}

#[test]
fn function_prototype_call() {
    let mut engine = Context::new();
    let func = r#"
        let e = new Error()
        Object.prototype.toString.call(e)
        "#;
    let value = forward_val(&mut engine, func).unwrap();
    assert!(value.is_string());
    assert_eq!(value.as_string().unwrap(), "[object Error]");
}
