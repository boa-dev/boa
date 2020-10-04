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
