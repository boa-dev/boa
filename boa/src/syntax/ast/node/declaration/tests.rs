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
    super::super::test_formatting(
        r#"
        let arrow_func = (a, b) => {
            console.log("in multi statement arrow");
            console.log(b);
        };
        async function async_func(a, b) {
            console.log(a);
        };
        pass_async_func(async function(a, b) {
            console.log("in async callback", a);
        });
        function func(a, b) {
            console.log(a);
        };
        pass_func(function(a, b) {
            console.log("in callback", a);
        });
        let arrow_func_2 = (a, b) => {};
        async function async_func_2(a, b) {};
        pass_async_func(async function(a, b) {});
        function func_2(a, b) {};
        pass_func(function(a, b) {});
        "#,
    );
}
