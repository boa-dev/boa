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
        pass_async_func(async function(a, b) {
            console.log("in async callback", a);
        });
        pass_func(function(a, b) {
            console.log("in callback", a);
        });
        let arrow_func_2 = (a, b) => {};
        async function async_func_2(a, b) {};
        pass_async_func(async function(a, b) {});
        pass_func(function(a, b) {});
        "#,
    );
}

#[test]
fn fmt_binding_pattern() {
    super::super::test_formatting(
        r#"
        var { } = {
            o: "1",
        };
        var { o_v1 } = {
            o_v1: "1",
        };
        var { o_v2 = "1" } = {
            o_v2: "2",
        };
        var { a : o_v3 = "1" } = {
            a: "2",
        };
        var { ... o_rest_v1 } = {
            a: "2",
        };
        var { o_v4, o_v5, o_v6 = "1", a : o_v7 = "1", ... o_rest_v2 } = {
            o_v4: "1",
            o_v5: "1",
        };
        var [] = [];
        var [ , ] = [];
        var [ a_v1 ] = [1, 2, 3];
        var [ a_v2, a_v3 ] = [1, 2, 3];
        var [ a_v2, , a_v3 ] = [1, 2, 3];
        var [ ... a_rest_v1 ] = [1, 2, 3];
        var [ a_v4, , ... a_rest_v2 ] = [1, 2, 3];
        var [ { a_v5 } ] = [{
            a_v5: 1,
        }, {
            a_v5: 2,
        }, {
            a_v5: 3,
        }];
        "#,
    );
}
