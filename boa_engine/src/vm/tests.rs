use crate::{check_output, exec, Context, JsValue, TestAction};

#[test]
fn typeof_string() {
    let typeof_object = r#"
        const a = "hello";
        typeof a;
    "#;
    assert_eq!(&exec(typeof_object), "\"string\"");
}

#[test]
fn typeof_number() {
    let typeof_number = r#"
        let a = 1234;
        typeof a;
    "#;
    assert_eq!(&exec(typeof_number), "\"number\"");
}

#[test]
fn basic_op() {
    let basic_op = r#"
        const a = 1;
        const b = 2;
        a + b
    "#;
    assert_eq!(&exec(basic_op), "3");
}

#[test]
fn try_catch_finally_from_init() {
    // the initialisation of the array here emits a PopOnReturnAdd op
    //
    // here we test that the stack is not popped more than intended due to multiple catches in the
    // same function, which could lead to VM stack corruption
    let source = r#"
        try {
            [(() => {throw "h";})()];
        } catch (x) {
            throw "h";
        } finally {
        }
    "#;

    assert_eq!(
        Context::default()
            .eval(source.as_bytes())
            .unwrap_err()
            .as_opaque()
            .unwrap(),
        &"h".into()
    );
}

#[test]
fn multiple_catches() {
    // see explanation on `try_catch_finally_from_init`
    let source = r#"
        try {
            try {
                [(() => {throw "h";})()];
            } catch (x) {
                throw "h";
            }
        } catch (y) {
        }
    "#;

    assert_eq!(
        Context::default().eval(source.as_bytes()).unwrap(),
        JsValue::Undefined
    );
}

#[test]
fn use_last_expr_try_block() {
    let source = r#"
        try {
            19;
            7.5;
            "Hello!";
        } catch (y) {
            14;
            "Bye!"
        }
    "#;

    assert_eq!(
        Context::default().eval(source.as_bytes()).unwrap(),
        JsValue::from("Hello!")
    );
}
#[test]
fn use_last_expr_catch_block() {
    let source = r#"
        try {
            throw Error("generic error");
            19;
            7.5;
        } catch (y) {
            14;
            "Hello!";
        }
    "#;

    assert_eq!(
        Context::default().eval(source.as_bytes()).unwrap(),
        JsValue::from("Hello!")
    );
}

#[test]
fn no_use_last_expr_finally_block() {
    let source = r#"
        try {
        } catch (y) {
        } finally {
            "Unused";
        }
    "#;

    assert_eq!(
        Context::default().eval(source.as_bytes()).unwrap(),
        JsValue::undefined()
    );
}

#[test]
fn finally_block_binding_env() {
    let source = r#"
        let buf = "Hey hey"
        try {
        } catch (y) {
        } finally {
            let x = " people";
            buf += x;
        }
        buf
    "#;

    assert_eq!(
        Context::default().eval(source.as_bytes()).unwrap(),
        JsValue::from("Hey hey people")
    );
}

#[test]
fn run_super_method_in_object() {
    let source = r#"
        let proto = {
            m() { return "super"; }
        };
        let obj = {
            v() { return super.m(); }
        };
        Object.setPrototypeOf(obj, proto);
        obj.v();
    "#;

    assert_eq!(
        Context::default().eval(source.as_bytes()).unwrap(),
        JsValue::from("super")
    );
}

#[test]
fn get_reference_by_super() {
    let source = r#"
        var fromA, fromB;
        var A = { fromA: 'a', fromB: 'a' };
        var B = { fromB: 'b' };
        Object.setPrototypeOf(B, A);
        var obj = {
            fromA: 'c',
            fromB: 'c',
            method() {
                fromA = (() => { return super.fromA; })();
                fromB = (() => { return super.fromB; })();
            }
        };
        Object.setPrototypeOf(obj, B);
        obj.method();
        fromA + fromB
    "#;

    assert_eq!(
        Context::default().eval(source.as_bytes()).unwrap(),
        JsValue::from("ab")
    );
}

#[test]
fn order_of_execution_in_assigment() {
    let scenario = r#"
        let i = 0;
        let array = [[]];

        array[i++][i++] = i++;
    "#;

    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("i", "3"),
        TestAction::TestEq("array.length", "1"),
        TestAction::TestEq("array[0].length", "2"),
    ]);
}

#[test]
fn order_of_execution_in_assigment_with_comma_expressions() {
    let scenario = r#"
        let result = "";
        function f(i) {
            result += i;
        }
        let a = [[]];
    
        (f(1), a)[(f(2), 0)][(f(3), 0)] = (f(4), 123);
    "#;

    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("result", "\"1234\""),
    ]);
}
