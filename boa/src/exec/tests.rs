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
    assert_eq!(&exec(scenario), "22");
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
    assert_eq!(&exec(element_changes), "5");

    let length_changes = r#"
        let m = [1, 2, 3];
        m[10] = 52;
        m.length
        "#;
    assert_eq!(&exec(length_changes), "11");

    let negative_index_wont_affect_length = r#"
        let m = [1, 2, 3];
        m[-11] = 5;
        m.length
        "#;
    assert_eq!(&exec(negative_index_wont_affect_length), "3");

    let non_num_key_wont_affect_length = r#"
        let m = [1, 2, 3];
        m["magic"] = 5;
        m.length
        "#;
    assert_eq!(&exec(non_num_key_wont_affect_length), "3");
}

#[test]
fn tilde_operator() {
    let float = r#"
        let f = -1.2;
        ~f
        "#;
    assert_eq!(&exec(float), "0");

    let numeric = r#"
        let f = 1789;
        ~f
        "#;
    assert_eq!(&exec(numeric), "-1790");

    // TODO: enable test after we have NaN
    // let nan = r#"
    // var m = NaN;
    // ~m
    // "#;
    // assert_eq!(&exec(nan), "-1");

    let object = r#"
        let m = {};
        ~m
        "#;
    assert_eq!(&exec(object), "-1");

    let boolean_true = r#"
        ~true
        "#;
    assert_eq!(&exec(boolean_true), "-2");

    let boolean_false = r#"
        ~false
        "#;
    assert_eq!(&exec(boolean_false), "-1");
}

#[test]
fn early_return() {
    let early_return = r#"
        function early_return() {
            if (true) {
                return true;
            }
            return false;
        }
        early_return()
        "#;
    assert_eq!(&exec(early_return), "true");

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
    assert_eq!(&exec(early_return), "outer");
}

#[test]
fn short_circuit_evaluation() {
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
    assert_eq!(&exec(short_circuit_eval), "1");

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
    assert_eq!(&exec(short_circuit_eval), "2");

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
    assert_eq!(&exec(short_circuit_eval), "2");

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
    assert_eq!(&exec(short_circuit_eval), "1");
}

#[test]
fn assign_operator_precedence() {
    let src = r#"
        let a = 1;
        a = a + 1;
        a
    "#;
    assert_eq!(&exec(src), "2");
}

#[test]
fn do_while_loop() {
    let simple_one = r#"
        a = 0;
        do {
            a += 1;
        } while (a < 10);

        a
         "#;
    assert_eq!(&exec(simple_one), "10");

    let multiline_statement = r#"
        pow = 0;
        b = 1;
        do {
            pow += 1;
            b *= 2;
        } while (pow < 8);
        b
        "#;
    assert_eq!(&exec(multiline_statement), "256");

    let body_is_executed_at_least_once = r#"
        a = 0;
        do
        {
            a += 1;
        }
        while (false);
        a
        "#;
    assert_eq!(&exec(body_is_executed_at_least_once), "1");
}

#[test]
fn do_while_post_inc() {
    let with_post_incrementors = r#"
        var i = 0;
        do {} while(i++ < 10) i;
    "#;
    assert_eq!(&exec(with_post_incrementors), "11");
}

#[test]
fn test_for_loop() {
    let simple = r#"
        const a = ['h', 'e', 'l', 'l', 'o'];
        let b = '';
        for (let i = 0; i < a.length; i++) {
            b = b + a[i];
        }
        b
        "#;
    assert_eq!(&exec(simple), "hello");

    let without_init_and_inc_step = r#"
        let a = 0;
        let i = 0;
        for (;i < 10;) {
            a = a + i;
            i++;
        }

        a
        "#;
    assert_eq!(&exec(without_init_and_inc_step), "45");

    let body_should_not_execute_on_false_condition = r#"
        let a = 0
        for (;false;) {
            a++;
        }

        a
        "#;
    assert_eq!(
        exec(body_should_not_execute_on_false_condition),
        String::from("0")
    );

    let inner_scope = r#"
        for (let i = 0;false;) {}

        i
        "#;
    assert_eq!(&exec(inner_scope), "undefined");
}

#[test]
fn unary_pre() {
    let unary_inc = r#"
        let a = 5;
        ++a;
        a;
    "#;
    assert_eq!(&exec(unary_inc), "6");

    let unary_dec = r#"
        let a = 5;
        --a;
        a;
    "#;
    assert_eq!(&exec(unary_dec), "4");

    let inc_obj_prop = r#"
        const a = { b: 5 };
        ++a.b;
        a['b'];
    "#;
    assert_eq!(&exec(inc_obj_prop), "6");

    let inc_obj_field = r#"
        const a = { b: 5 };
        ++a['b'];
        a.b;
    "#;
    assert_eq!(&exec(inc_obj_field), "6");

    let execs_before_inc = r#"
        let a = 5;
        ++a === 6;
    "#;
    assert_eq!(&exec(execs_before_inc), "true");

    let execs_before_dec = r#"
        let a = 5;
        --a === 4;
    "#;
    assert_eq!(&exec(execs_before_dec), "true");
}

#[test]
fn unary_post() {
    let unary_inc = r#"
        let a = 5;
        a++;
        a;
    "#;
    assert_eq!(&exec(unary_inc), "6");

    let unary_dec = r#"
        let a = 5;
        a--;
        a;
    "#;
    assert_eq!(&exec(unary_dec), "4");

    let inc_obj_prop = r#"
        const a = { b: 5 };
        a.b++;
        a['b'];
    "#;
    assert_eq!(&exec(inc_obj_prop), "6");

    let inc_obj_field = r#"
        const a = { b: 5 };
        a['b']++;
        a.b;
    "#;
    assert_eq!(&exec(inc_obj_field), "6");

    let execs_after_inc = r#"
        let a = 5;
        a++ === 5;
    "#;
    assert_eq!(&exec(execs_after_inc), "true");

    let execs_after_dec = r#"
        let a = 5;
        a-- === 5;
    "#;
    assert_eq!(&exec(execs_after_dec), "true");
}

#[test]
fn unary_void() {
    let void_should_return_undefined = r#"
        const a = 0;
        void a;
    "#;
    assert_eq!(&exec(void_should_return_undefined), "undefined");

    let void_invocation = r#"
        let a = 0;
        const test = () => a = 42;
        const b = void test() + '';
        a + b
    "#;
    assert_eq!(&exec(void_invocation), "42undefined");
}

#[test]
fn unary_delete() {
    let delete_var = r#"
        let a = 5;
        const b = delete a + '';
        a + b
    "#;
    assert_eq!(&exec(delete_var), "5false");

    let delete_prop = r#"
        const a = { b: 5 };
        const c = delete a.b + '';
        a.b + c
    "#;
    assert_eq!(&exec(delete_prop), "undefinedtrue");

    let delete_not_existing_prop = r#"
        const a = { b: 5 };
        const c = delete a.c + '';
        a.b + c
    "#;
    assert_eq!(&exec(delete_not_existing_prop), "5false");

    let delete_field = r#"
        const a = { b: 5 };
        const c = delete a['b'] + '';
        a.b + c
    "#;
    assert_eq!(&exec(delete_field), "undefinedtrue");

    let delete_object = r#"
        const a = { b: 5 };
        delete a
    "#;
    assert_eq!(&exec(delete_object), "false");

    let delete_array = r#"
        delete [];
    "#;
    assert_eq!(&exec(delete_array), "true");

    let delete_func = r#"
        delete function() {};
    "#;
    assert_eq!(&exec(delete_func), "true");

    let delete_recursive = r#"
        delete delete delete 1;
    "#;
    assert_eq!(&exec(delete_recursive), "true");
}

#[cfg(test)]
mod in_operator {
    use super::*;
    #[test]
    fn propery_in_object() {
        let p_in_o = r#"
            var o = {a: 'a'};
            var p = 'a';
            p in o
        "#;
        assert_eq!(&exec(p_in_o), "true");
    }

    #[test]
    fn property_in_property_chain() {
        let p_in_o = r#"
            var o = {};
            var p = 'toString';
            p in o
        "#;
        assert_eq!(&exec(p_in_o), "true");
    }

    #[test]
    fn property_not_in_object() {
        let p_not_in_o = r#"
            var o = {a: 'a'};
            var p = 'b';
            p in o
        "#;
        assert_eq!(&exec(p_not_in_o), "false");
    }

    #[test]
    fn number_in_array() {
        // Note: this is valid because the LHS is converted to a prop key with ToPropertyKey
        // and arrays are just fancy objects like {'0': 'a'}
        let num_in_array = r#"
            var n = 0;
            var a = ['a'];
            n in a
        "#;
        assert_eq!(&exec(num_in_array), "true");
    }

    #[test]
    #[ignore]
    fn symbol_in_object() {
        // FIXME: this scenario works in Firefox's console, this is probably an issue
        // with Symbol comparison.
        let sym_in_object = r#"
            var sym = Symbol('hi');
            var o = {};
            o[sym] = 'hello';
            sym in o
        "#;
        assert_eq!(&exec(sym_in_object), "true");
    }

    #[test]
    #[should_panic(expected = "TypeError: undefined is not an Object.")]
    fn should_type_error_when_rhs_not_object() {
        let scenario = r#"
            'fail' in undefined
        "#;
        exec(scenario);
    }
}
