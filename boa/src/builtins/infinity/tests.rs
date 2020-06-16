use crate::exec;

#[test]
fn infinity_exists_on_global_object_and_evaluates_to_infinity_value() {
    let scenario = r#"
        Infinity;
        "#;

    assert_eq!(&exec(scenario), "Infinity");
}

#[test]
fn infinity_exists_and_equals_to_number_positive_infinity_value() {
    let scenario = r#"
        Number.POSITIVE_INFINITY === Infinity;
        "#;

    assert_eq!(&exec(scenario), "true");
}
