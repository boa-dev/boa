use crate::exec;
use crate::exec::Executor;
use crate::forward;
use crate::realm::Realm;

#[test]
fn empty_let_decl_undefined() {
    let scenario = r#"
        let a;
        a == undefined;
        "#;

    let pass = String::from("true");

    assert_eq!(exec(scenario), pass);
}

#[test]
fn semicolon_expression_stop() {
    let scenario = r#"
        var a = 1;
        + 1;
        a
        "#;

    let pass = String::from("1");

    assert_eq!(exec(scenario), pass);
}

#[test]
fn empty_var_decl_undefined() {
    let scenario = r#"
        let b;
        b == undefined;
        "#;

    let pass = String::from("true");

    assert_eq!(exec(scenario), pass);
}

#[test]
fn object_field_set() {
    let scenario = r#"
        let m = {};
        m['key'] = 22;
        m['key']
        "#;
    assert_eq!(exec(scenario), String::from("22"));
}

#[test]
fn spread_with_arguments() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);

    let scenario = r#"
            const a = [1, "test", 3, 4];
            function foo(...a) {
                return arguments;
            }
            
            var result = foo(...a);
        "#;
    forward(&mut engine, scenario);
    let one = forward(&mut engine, "result[0]");
    assert_eq!(one, String::from("1"));

    let two = forward(&mut engine, "result[1]");
    assert_eq!(two, String::from("test"));

    let three = forward(&mut engine, "result[2]");
    assert_eq!(three, String::from("3"));

    let four = forward(&mut engine, "result[3]");
    assert_eq!(four, String::from("4"));
}

#[test]
fn array_rest_with_arguments() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);

    let scenario = r#"
            var b = [4, 5, 6]
            var a = [1, 2, 3, ...b];
        "#;
    forward(&mut engine, scenario);
    let one = forward(&mut engine, "a");
    assert_eq!(one, String::from("[ 1, 2, 3, 4, 5, 6 ]"));
}

#[test]
fn array_field_set() {
    let element_changes = r#"
        let m = [1, 2, 3];
        m[1] = 5;
        m[1]
        "#;
    assert_eq!(exec(element_changes), String::from("5"));

    let length_changes = r#"
        let m = [1, 2, 3];
        m[10] = 52;
        m.length
        "#;
    assert_eq!(exec(length_changes), String::from("11"));

    let negative_index_wont_affect_length = r#"
        let m = [1, 2, 3];
        m[-11] = 5;
        m.length
        "#;
    assert_eq!(exec(negative_index_wont_affect_length), String::from("3"));

    let non_num_key_wont_affect_length = r#"
        let m = [1, 2, 3];
        m["magic"] = 5;
        m.length
        "#;
    assert_eq!(exec(non_num_key_wont_affect_length), String::from("3"));
}

#[test]
fn test_tilde_operator() {
    let float = r#"
        let f = -1.2;
        ~f
        "#;
    assert_eq!(exec(float), String::from("0"));

    let numeric = r#"
        let f = 1789;
        ~f
        "#;
    assert_eq!(exec(numeric), String::from("-1790"));

    // TODO: enable test after we have NaN
    // let nan = r#"
    // var m = NaN;
    // ~m
    // "#;
    // assert_eq!(exec(nan), String::from("-1"));

    let object = r#"
        let m = {};
        ~m
        "#;
    assert_eq!(exec(object), String::from("-1"));

    let boolean_true = r#"
        ~true
        "#;
    assert_eq!(exec(boolean_true), String::from("-2"));

    let boolean_false = r#"
        ~false
        "#;
    assert_eq!(exec(boolean_false), String::from("-1"));
}

#[test]
fn test_early_return() {
    let early_return = r#"
        function early_return() {
            if (true) {
                return true;
            }
            return false;
        }
        early_return()
        "#;
    assert_eq!(exec(early_return), String::from("true"));
    let early_return = r#"
        function nested_fnct() {
            return "nested";
        }
        function outer_fnct() {
            nested_fnct();
            return "outer";
        }
        outer_fnct()
        "#;
    assert_eq!(exec(early_return), String::from("outer"));
}

#[test]
fn test_short_circuit_evaluation() {
    // OR operation
    assert_eq!(exec("true || true"), String::from("true"));
    assert_eq!(exec("true || false"), String::from("true"));
    assert_eq!(exec("false || true"), String::from("true"));
    assert_eq!(exec("false || false"), String::from("false"));

    // the second operand must NOT be evaluated if the first one resolve to `true`.
    let short_circuit_eval = r#"
        function add_one(counter) {
            counter.value += 1;
            return true;
        }
        let counter = { value: 0 };
        let _ = add_one(counter) || add_one(counter);
        counter.value
        "#;
    assert_eq!(exec(short_circuit_eval), String::from("1"));

    // the second operand must be evaluated if the first one resolve to `false`.
    let short_circuit_eval = r#"
        function add_one(counter) {
            counter.value += 1;
            return false;
        }
        let counter = { value: 0 };
        let _ = add_one(counter) || add_one(counter);
        counter.value
        "#;
    assert_eq!(exec(short_circuit_eval), String::from("2"));

    // AND operation
    assert_eq!(exec("true && true"), String::from("true"));
    assert_eq!(exec("true && false"), String::from("false"));
    assert_eq!(exec("false && true"), String::from("false"));
    assert_eq!(exec("false && false"), String::from("false"));

    // the second operand must be evaluated if the first one resolve to `true`.
    let short_circuit_eval = r#"
        function add_one(counter) {
            counter.value += 1;
            return true;
        }
        let counter = { value: 0 };
        let _ = add_one(counter) && add_one(counter);
        counter.value
        "#;
    assert_eq!(exec(short_circuit_eval), String::from("2"));

    // the second operand must NOT be evaluated if the first one resolve to `false`.
    let short_circuit_eval = r#"
        function add_one(counter) {
            counter.value += 1;
            return false;
        }
        let counter = { value: 0 };
        let _ = add_one(counter) && add_one(counter);
        counter.value
        "#;
    assert_eq!(exec(short_circuit_eval), String::from("1"));
}
