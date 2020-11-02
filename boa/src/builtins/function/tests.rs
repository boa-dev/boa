use crate::{forward, forward_val, Context};

#[allow(clippy::float_cmp)]
#[test]
fn arguments_object() {
    let mut context = Context::new();

    let init = r#"
        function jason(a, b) {
            return arguments[0];
        }
        var val = jason(100, 6);
        "#;

    eprintln!("{}", forward(&mut context, init));

    let return_val = forward_val(&mut context, "val").expect("value expected");
    assert_eq!(return_val.is_integer(), true);
    assert_eq!(
        return_val
            .to_i32(&mut context)
            .expect("Could not convert value to i32"),
        100
    );
}

#[test]
fn self_mutating_function_when_calling() {
    let mut context = Context::new();
    let func = r#"
        function x() {
	        x.y = 3;
        }
        x();
        "#;
    eprintln!("{}", forward(&mut context, func));
    let y = forward_val(&mut context, "x.y").expect("value expected");
    assert_eq!(y.is_integer(), true);
    assert_eq!(
        y.to_i32(&mut context)
            .expect("Could not convert value to i32"),
        3
    );
}

#[test]
fn self_mutating_function_when_constructing() {
    let mut context = Context::new();
    let func = r#"
        function x() {
            x.y = 3;
        }
        new x();
        "#;
    eprintln!("{}", forward(&mut context, func));
    let y = forward_val(&mut context, "x.y").expect("value expected");
    assert_eq!(y.is_integer(), true);
    assert_eq!(
        y.to_i32(&mut context)
            .expect("Could not convert value to i32"),
        3
    );
}

#[test]
fn call_function_prototype() {
    let mut context = Context::new();
    let func = r#"
        Function.prototype()
        "#;
    let value = forward_val(&mut context, func).unwrap();
    assert!(value.is_undefined());
}

#[test]
fn call_function_prototype_with_arguments() {
    let mut context = Context::new();
    let func = r#"
        Function.prototype(1, "", new String(""))
        "#;
    let value = forward_val(&mut context, func).unwrap();
    assert!(value.is_undefined());
}

#[test]
fn call_function_prototype_with_new() {
    let mut context = Context::new();
    let func = r#"
        new Function.prototype()
        "#;
    let value = forward_val(&mut context, func);
    assert!(value.is_err());
}

#[test]
fn function_prototype_name() {
    let mut context = Context::new();
    let func = r#"
        Function.prototype.name
        "#;
    let value = forward_val(&mut context, func).unwrap();
    assert!(value.is_string());
    assert!(value.as_string().unwrap().is_empty());
}

#[test]
#[allow(clippy::float_cmp)]
fn function_prototype_length() {
    let mut context = Context::new();
    let func = r#"
        Function.prototype.length
        "#;
    let value = forward_val(&mut context, func).unwrap();
    assert!(value.is_number());
    assert_eq!(value.as_number().unwrap(), 0.0);
}

#[test]
fn function_prototype_call() {
    let mut context = Context::new();
    let func = r#"
        let e = new Error()
        Object.prototype.toString.call(e)
        "#;
    let value = forward_val(&mut context, func).unwrap();
    assert!(value.is_string());
    assert_eq!(value.as_string().unwrap(), "[object Error]");
}

#[test]
fn function_prototype_call_throw() {
    let mut context = Context::new();
    let throw = r#"
        let call = Function.prototype.call;
        call(call)
        "#;
    let value = forward_val(&mut context, throw).unwrap_err();
    assert!(value.is_object());
    let string = value.to_string(&mut context).unwrap();
    assert!(string.starts_with("TypeError"))
}

#[test]
fn function_prototype_call_multiple_args() {
    let mut context = Context::new();
    let init = r#"
        function f(a, b) {
            this.a = a;
            this.b = b;
        }
        let o = {a: 0, b: 0};
        f.call(o, 1, 2);
    "#;
    forward_val(&mut context, init).unwrap();
    let boolean = forward_val(&mut context, "o.a == 1")
        .unwrap()
        .as_boolean()
        .unwrap();
    assert!(boolean);
    let boolean = forward_val(&mut context, "o.b == 2")
        .unwrap()
        .as_boolean()
        .unwrap();
    assert!(boolean);
}

#[test]
fn function_prototype_apply() {
    let mut context = Context::new();
    let init = r#"
        const numbers = [6, 7, 3, 4, 2];
        const max = Math.max.apply(null, numbers);
        const min = Math.min.apply(null, numbers);
    "#;
    forward_val(&mut context, init).unwrap();

    let boolean = forward_val(&mut context, "max == 7")
        .unwrap()
        .as_boolean()
        .unwrap();
    assert!(boolean);

    let boolean = forward_val(&mut context, "min == 2")
        .unwrap()
        .as_boolean()
        .unwrap();
    assert!(boolean);
}

#[test]
fn function_prototype_apply_on_object() {
    let mut context = Context::new();
    let init = r#"
        function f(a, b) {
            this.a = a;
            this.b = b;
        }
        let o = {a: 0, b: 0};
        f.apply(o, [1, 2]);
    "#;
    forward_val(&mut context, init).unwrap();

    let boolean = forward_val(&mut context, "o.a == 1")
        .unwrap()
        .as_boolean()
        .unwrap();
    assert!(boolean);

    let boolean = forward_val(&mut context, "o.b == 2")
        .unwrap()
        .as_boolean()
        .unwrap();
    assert!(boolean);
}
