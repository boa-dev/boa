use crate::{forward, forward_val, Context};

#[allow(clippy::float_cmp)]
#[test]
fn check_arguments_object() {
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
fn check_self_mutating_func() {
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
