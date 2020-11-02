use crate::exec;

#[test]
fn ordinary_has_instance_nonobject_prototype() {
    let scenario = r#"
        try {
          function C() {}
          C.prototype = 1
          String instanceof C
        } catch (err) {
          err.toString()
        }
        "#;

    assert_eq!(
        &exec(scenario),
        "\"TypeError: function has non-object prototype in instanceof check\""
    );
}
