use crate::exec;

#[test]
fn global_this_exists_on_global_object_and_evaluates_to_global_this_value() {
    let scenario = r#"
        globalThis;
        "#;

    assert_eq!(&exec(scenario), "globalThis");
}
