use crate::exec;

#[test]
fn spread_with_new() {
    let scenario = r#"
    function F(m) {
        this.m = m;
    }
    function f(...args) {
        return new F(...args);
    }
    let a = f('message');
    a.m;
    "#;
    assert_eq!(&exec(scenario), r#""message""#);
}

#[test]
fn spread_with_call() {
    let scenario = r#"
    function f(m) {
        return m;
    }
    function g(...args) {
        return f(...args);
    }
    let a = g('message');
    a;
    "#;
    assert_eq!(&exec(scenario), r#""message""#);
}

#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        function f(m) {
            return m;
        };
        function g(...args) {
            return f(...args);
        };
        let a = g("message");
        a;
        "#,
    );
}
