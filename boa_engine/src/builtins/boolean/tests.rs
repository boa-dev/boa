use crate::{forward, forward_val, Context};

/// Test the correct type is returned from call and construct
#[allow(clippy::unwrap_used)]
#[test]
fn construct_and_call() {
    let mut context = Context::default();
    let init = r#"
        var one = new Boolean(1);
        var zero = Boolean(0);
        "#;
    eprintln!("{}", forward(&mut context, init));
    let one = forward_val(&mut context, "one").unwrap();
    let zero = forward_val(&mut context, "zero").unwrap();

    assert!(one.is_object());
    assert!(zero.is_boolean());
}

#[test]
fn constructor_gives_true_instance() {
    let mut context = Context::default();
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
    assert!(true_val.is_object());
    assert!(true_num.is_object());
    assert!(true_string.is_object());
    assert!(true_bool.is_object());

    // Values should all be truthy
    assert!(true_val.to_boolean());
    assert!(true_num.to_boolean());
    assert!(true_string.to_boolean());
    assert!(true_bool.to_boolean());
}

#[test]
fn instances_have_correct_proto_set() {
    let mut context = Context::default();
    let init = r#"
        var boolInstance = new Boolean(true);
        var boolProto = Boolean.prototype;
        "#;

    eprintln!("{}", forward(&mut context, init));
    let bool_instance = forward_val(&mut context, "boolInstance").expect("value expected");
    let bool_prototype = forward_val(&mut context, "boolProto").expect("value expected");

    assert_eq!(
        &*bool_instance.as_object().unwrap().prototype(),
        &bool_prototype.as_object()
    );
}
