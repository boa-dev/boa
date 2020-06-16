use crate::exec;

#[test]
fn undefined_direct_evaluation() {
    let scenario = r#"
        undefined;
        "#;
    assert_eq!(&exec(scenario), "undefined");
}

#[test]
fn undefined_assignment() {
    let scenario = r#"
        a = undefined;
        a
        "#;
    assert_eq!(&exec(scenario), "undefined");
}
