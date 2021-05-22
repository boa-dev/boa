use crate::{forward, forward_val, Context, Value};

/// Test the correct type is returned from call and construct
#[allow(clippy::unwrap_used)]
#[test]
fn construct_and_call() {
    let mut context = Context::new();
    let init = r#"
        var one = new Boolean(1);
        var zero = Boolean(0);
        "#;
    eprintln!("{}", forward(&mut context, init));
    let one = forward_val(&mut context, "one").unwrap();
    let zero = forward_val(&mut context, "zero").unwrap();

    assert_eq!(one.is_object(), true);
    assert_eq!(zero.is_boolean(), true);
}

#[test]
fn constructor_gives_true_instance() {
    let mut context = Context::new();
    let init = r#"
        var trueVal = new Boolean(true);
        var trueNum = new Boolean(1);
        var trueString = new Boolean("true");
        var trueBool = new Boolean(trueVal);
        "#;

    eprintln!("{}", forward(&mut context, init));
    let true_val = forward_val(&mut context, "trueVal").expect("value expected");
    let true_num = forward_val(&mut context, "trueNum").expect("value expected");
    let true_string = forward_val(&mut context, "trueString").expect("value expected");
    let true_bool = forward_val(&mut context, "trueBool").expect("value expected");

    // Values should all be objects
    assert_eq!(true_val.is_object(), true);
    assert_eq!(true_num.is_object(), true);
    assert_eq!(true_string.is_object(), true);
    assert_eq!(true_bool.is_object(), true);

    // Values should all be truthy
    assert_eq!(true_val.to_boolean(), true);
    assert_eq!(true_num.to_boolean(), true);
    assert_eq!(true_string.to_boolean(), true);
    assert_eq!(true_bool.to_boolean(), true);
}

#[test]
fn instances_have_correct_proto_set() {
    let mut context = Context::new();
    let init = r#"
        var boolInstance = new Boolean(true);
        var boolProto = Boolean.prototype;
        "#;

    eprintln!("{}", forward(&mut context, init));
    let bool_instance = forward_val(&mut context, "boolInstance").expect("value expected");
    let bool_prototype = forward_val(&mut context, "boolProto").expect("value expected");

    assert!(Value::same_value(
        &bool_instance.as_object().unwrap().prototype_instance(),
        &bool_prototype
    ));
}
