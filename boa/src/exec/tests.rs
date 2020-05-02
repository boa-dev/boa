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

#[test]
fn assign_operator_precedence() {
    let src = r#"
        let a = 1;
        a = a + 1;
        a
    "#;
    assert_eq!(exec(src), String::from("2"));
}

#[test]
fn test_do_while_loop() {
    let simple_one = r#"
        a = 0;
        do {
            a += 1;
        } while (a < 10);

        a
         "#;
    assert_eq!(exec(simple_one), String::from("10"));

    let multiline_statement = r#"
        pow = 0;
        b = 1;
        do {
            pow += 1;
            b *= 2;
        } while (pow < 8);
        b
        "#;
    assert_eq!(exec(multiline_statement), String::from("256"));

    let body_is_executed_at_least_once = r#"
        a = 0;
        do
        {
            a += 1;
        }
        while (false);
        a
        "#;
    assert_eq!(exec(body_is_executed_at_least_once), String::from("1"));
}

#[test]
#[ignore]
fn test_do_while_post_inc() {
    let with_post_incrementors = r#"
        var i = 0;
        do {} while(i++ < 10) i;
    "#;
    assert_eq!(exec(with_post_incrementors), String::from("11"));
}

#[test]
#[ignore]
fn test_unary_pre() {
    let unary_inc = r#"
        let a = 5;
        ++a;
        a;
    "#;
    assert_eq!(exec(unary_inc), String::from("6"));

    let unary_dec = r#"
        let a = 5;
        --a;
        a;
    "#;
    assert_eq!(exec(unary_dec), String::from("6"));

    let execs_before = r#"
        let a = 5;
        ++a === 6;
    "#;
    assert_eq!(exec(execs_before), String::from("true"));
}

#[test]
#[ignore]
fn test_unary_post() {
    let unary_inc = r#"
        let a = 5;
        a++;
        a;
    "#;
    assert_eq!(exec(unary_inc), String::from("6"));

    let unary_dec = r#"
        let a = 5;
        a--;
        a;
    "#;
    assert_eq!(exec(unary_dec), String::from("6"));

    let execs_after = r#"
        let a = 5;
        a++ === 5;
    "#;
    assert_eq!(exec(execs_after), String::from("true"));
}

#[cfg(test)]
mod in_operator {
    use super::*;
    #[test]
    fn test_p_in_o() {
        let p_in_o = r#"
            var o = {a: 'a'};
            var p = 'a';
            p in o
        "#;
        assert_eq!(exec(p_in_o), String::from("true"));
    }

    #[test]
    fn test_p_in_property_chain() {
        let p_in_o = r#"
            var o = {};
            var p = 'toString';
            p in o
        "#;
        assert_eq!(exec(p_in_o), String::from("true"));
    }

    #[test]
    fn test_p_not_in_o() {
        let p_not_in_o = r#"
            var o = {a: 'a'};
            var p = 'b';
            p in o
        "#;
        assert_eq!(exec(p_not_in_o), String::from("false"));
    }

    #[test]
    fn test_number_in_array() {
        // Note: this is valid because the LHS is converted to a prop key with ToPropertyKey
        // and arrays are just fancy objects like {'0': 'a'}
        let num_in_array = r#"
            var n = 0;
            var a = ['a'];
            n in a
        "#;
        assert_eq!(exec(num_in_array), String::from("true"));
    }

    #[test]
    #[ignore]
    fn test_symbol_in_object() {
        // FIXME: this scenario works in Firefox's console, this is probably an issue
        // with Symbol comparison.
        let sym_in_object = r#"
            var sym = Symbol('hi');
            var o = {};
            o[sym] = 'hello';
            sym in o
        "#;
        assert_eq!(exec(sym_in_object), String::from("true"));
    }

    #[test]
    #[should_panic(expected = "TypeError: undefined is not an Object.")]
    fn test_should_type_error_when_rhs_not_object() {
        let scenario = r#"
            'fail' in undefined
        "#;
        exec(scenario);
    }
}
