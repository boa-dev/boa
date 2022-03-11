use crate::{check_output, exec, TestAction};

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

#[test]
fn object_properties_return_order() {
    let scenario = r#"
        var o = {
            p1: 'v1',
            p2: 'v2',
            p3: 'v3',
        };
        o.p4 = 'v4';
        o[2] = 'iv2';
        o[0] = 'iv0';
        o[1] = 'iv1';
        delete o.p1;
        delete o.p3;
        o.p1 = 'v1';
        "#;

    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("Object.keys(o)", r#"[ "0", "1", "2", "p2", "p4", "p1" ]"#),
        TestAction::TestEq(
            "Object.values(o)",
            r#"[ "iv0", "iv1", "iv2", "v2", "v4", "v1" ]"#,
        ),
    ]);
}
