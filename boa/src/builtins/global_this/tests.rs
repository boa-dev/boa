use crate::exec;

#[test]
fn global_this_exists_on_global_object_and_evaluates_to_an_object() {
    let scenario = r#"
        typeof globalThis;
        "#;

    assert_eq!(&exec(scenario), "\"object\"");
}
