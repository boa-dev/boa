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

    forward(&mut engine, init);
    let expected_return_val: f64 = 100.0;
    let return_val = forward_val(&mut engine, "val").expect("value expected");
    assert_eq!(return_val.is_double(), true);
    assert_eq!(
        from_value::<f64>(return_val).expect("Could not convert value to f64"),
        expected_return_val
    );
}
