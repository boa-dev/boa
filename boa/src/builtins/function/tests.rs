use crate::exec::Executor;
use crate::realm::Realm;
use crate::{builtins::value::from_value, forward, forward_val};

#[allow(clippy::float_cmp)]
#[test]
fn check_arguments_object() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        function jason(a, b) {
            return arguments[0];
        }
        var val = jason(100, 6);
        "#;

    eprintln!("{}", forward(&mut engine, init));
    let expected_return_val = 100;
    let return_val = forward_val(&mut engine, "val").expect("value expected");
    assert_eq!(return_val.is_integer(), true);
    assert_eq!(
        from_value::<i32>(return_val).expect("Could not convert value to i32"),
        expected_return_val
    );
}
