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

#[test]
fn fmt() {
    // TODO: Async function expr are considered valid syntax, but are converted
    // into normal functions somewhere in the parser.
    super::super::test_formatting(
        r#"
        function func(a, b) {
            console.log(a);
        };
        function func_2(a, b) {};
        let arrow_func = (a, b) => {
            console.log("in multi statement arrow");
            console.log(b);
        };
        async function async_func(a, b) {
            console.log(a);
        };
        pass_async_func(function(a, b) {
            console.log("in async callback", a);
        });
        pass_func(function(a, b) {
            console.log("in callback", a);
        });
        let arrow_func_2 = (a, b) => {};
        async function async_func_2(a, b) {};
        pass_async_func(function(a, b) {});
        pass_func(function(a, b) {});
        "#,
    );
}
