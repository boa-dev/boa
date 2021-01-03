use crate::exec;

#[test]
fn duplicate_function_name() {
    let scenario = r#"
    function f () {}
    function f () {return 12;}
    f()
    "#;

    assert_eq!(&exec(scenario), "12");
}
