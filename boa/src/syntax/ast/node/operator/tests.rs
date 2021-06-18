use crate::exec;

#[test]
fn assignmentoperator_lhs_not_defined() {
    let scenario = r#"
        try {
          a += 5
        } catch (err) {
          err.toString()
        }
        "#;

    assert_eq!(&exec(scenario), "\"ReferenceError: a is not defined\"");
}

#[test]
fn assignmentoperator_rhs_throws_error() {
    let scenario = r#"
        try {
          let a;
          a += b
        } catch (err) {
          err.toString()
        }
        "#;

    assert_eq!(&exec(scenario), "\"ReferenceError: b is not defined\"");
}

#[test]
fn instanceofoperator_rhs_not_object() {
    let scenario = r#"
        try {
          let s = new String();
          s instanceof 1
        } catch (err) {
          err.toString()
        }
        "#;

    assert_eq!(
        &exec(scenario),
        "\"TypeError: right-hand side of 'instanceof' should be an object, got number\""
    );
}

#[test]
fn instanceofoperator_rhs_not_callable() {
    let scenario = r#"
        try {
          let s = new String();
          s instanceof {}
        } catch (err) {
          err.toString()
        }
        "#;

    assert_eq!(
        &exec(scenario),
        "\"TypeError: right-hand side of 'instanceof' is not callable\""
    );
}

#[test]
fn logical_nullish_assignment() {
    let scenario = r#"
        let a = undefined;
        a ??= 10;
        a;
        "#;

    assert_eq!(&exec(scenario), "10");

    let scenario = r#"
        let a = 20;
        a ??= 10;
        a;
        "#;

    assert_eq!(&exec(scenario), "20");
}

#[test]
fn logical_assignment() {
    let scenario = r#"
        let a = false;
        a &&= 10;
        a;
        "#;

    assert_eq!(&exec(scenario), "false");

    let scenario = r#"
        let a = 20;
        a &&= 10;
        a;
        "#;

    assert_eq!(&exec(scenario), "10");

    let scenario = r#"
        let a = null;
        a ||= 10;
        a;
        "#;

    assert_eq!(&exec(scenario), "10");
    let scenario = r#"
        let a = 20;
        a ||= 10;
        a;
        "#;

    assert_eq!(&exec(scenario), "20");
}

#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        let a = 20;
        a += 10;
        a -= 10;
        a *= 10;
        a **= 10;
        a /= 10;
        a %= 10;
        a &= 10;
        a |= 10;
        a ^= 10;
        a <<= 10;
        a >>= 10;
        a >>>= 10;
        a &&= 10;
        a ||= 10;
        a ??= 10;
        a;
        "#,
    );
}
