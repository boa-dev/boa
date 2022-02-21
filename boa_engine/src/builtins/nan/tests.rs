use crate::exec;

#[test]
fn nan_exists_on_global_object_and_evaluates_to_nan_value() {
    let scenario = r#"
        NaN;
        "#;

    assert_eq!(&exec(scenario), "NaN");
}
