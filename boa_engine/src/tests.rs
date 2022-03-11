use crate::{
    builtins::Number, check_output, exec, forward, forward_val, value::IntegerOrInfinity, Context,
    JsValue, TestAction,
};

#[test]
fn function_declaration_returns_undefined() {
    let scenario = r#"
        function abc() {}
        "#;

    assert_eq!(&exec(scenario), "undefined");
}

#[test]
fn empty_function_returns_undefined() {
    let scenario = "(function () {}) ()";
    assert_eq!(&exec(scenario), "undefined");
}

#[test]
fn property_accessor_member_expression_dot_notation_on_string_literal() {
    let scenario = r#"
        typeof 'asd'.matchAll;
        "#;

    assert_eq!(&exec(scenario), "\"function\"");
}

#[test]
fn property_accessor_member_expression_bracket_notation_on_string_literal() {
    let scenario = r#"
        typeof 'asd'['matchAll'];
        "#;

    assert_eq!(&exec(scenario), "\"function\"");
}

#[test]
fn length_correct_value_on_string_literal() {
    let scenario = r#"
    'hello'.length;
    "#;

    assert_eq!(&exec(scenario), "5");
}

#[test]
fn property_accessor_member_expression_dot_notation_on_function() {
    let scenario = r#"
        function asd () {};
        asd.name;
        "#;

    assert_eq!(&exec(scenario), "\"asd\"");
}

#[test]
fn property_accessor_member_expression_bracket_notation_on_function() {
    let scenario = r#"
        function asd () {};
        asd['name'];
        "#;

    assert_eq!(&exec(scenario), "\"asd\"");
}

#[test]
fn empty_let_decl_undefined() {
    let scenario = r#"
        let a;
        a === undefined;
        "#;

    assert_eq!(&exec(scenario), "true");
}

#[test]
fn semicolon_expression_stop() {
    let scenario = r#"
        var a = 1;
        + 1;
        a
        "#;

    assert_eq!(&exec(scenario), "1");
}

#[test]
fn empty_var_decl_undefined() {
    let scenario = r#"
        let b;
        b === undefined;
        "#;

    assert_eq!(&exec(scenario), "true");
}

#[test]
fn identifier_on_global_object_undefined() {
    let scenario = r#"
        try {
            bar;
        } catch (err) {
            err.message
        }
        "#;

    assert_eq!(&exec(scenario), "\"bar is not defined\"");
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
fn object_spread() {
    let scenario = r#"
            var b = {x: -1, z: -3}
            var a = {x: 1, y: 2, ...b};
        "#;

    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("a.x", "-1"),
        TestAction::TestEq("a.y", "2"),
        TestAction::TestEq("a.z", "-3"),
    ]);
}

#[test]
fn spread_with_arguments() {
    let scenario = r#"
            const a = [1, "test", 3, 4];
            function foo(...a) {
                return arguments;
            }

            var result = foo(...a);
        "#;

    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("result[0]", "1"),
        TestAction::TestEq("result[1]", "\"test\""),
        TestAction::TestEq("result[2]", "3"),
        TestAction::TestEq("result[3]", "4"),
    ]);
}

#[test]
fn array_rest_with_arguments() {
    let scenario = r#"
                var b = [4, 5, 6]
                var a = [1, 2, 3, ...b];
            "#;

    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("a", "[ 1, 2, 3, 4, 5, 6 ]"),
    ]);
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

    let nan = r#"
    var m = NaN;
    ~m
    "#;
    assert_eq!(&exec(nan), "-1");

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
    assert_eq!(&exec(early_return), "\"outer\"");
}

#[test]
fn short_circuit_evaluation() {
    // OR operation
    assert_eq!(&exec("true || true"), "true");
    assert_eq!(&exec("true || false"), "true");
    assert_eq!(&exec("false || true"), "true");
    assert_eq!(&exec("false || false"), "false");

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
    assert_eq!(&exec("true && true"), "true");
    assert_eq!(&exec("true && false"), "false");
    assert_eq!(&exec("false && true"), "false");
    assert_eq!(&exec("false && false"), "false");

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
}

#[test]
fn do_while_loop_at_least_once() {
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
fn for_loop() {
    let simple = r#"
        const a = ['h', 'e', 'l', 'l', 'o'];
        let b = '';
        for (let i = 0; i < a.length; i++) {
            b = b + a[i];
        }
        b
        "#;
    assert_eq!(&exec(simple), "\"hello\"");

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
    assert_eq!(&exec(body_should_not_execute_on_false_condition), "0");
}

#[test]
fn for_loop_iteration_variable_does_not_leak() {
    let inner_scope = r#"
        for (let i = 0;false;) {}

        try {
            i
        } catch (err) {
            err.message
        }
        "#;

    assert_eq!(&exec(inner_scope), "\"i is not defined\"");
}

#[test]
fn test_invalid_break_target() {
    let src = r#"
        while (false) {
          break nonexistent;
        }
        "#;

    assert!(Context::default().eval(src).is_err());
}

#[test]
fn test_invalid_continue_target() {
    let mut context = Context::default();
    let src = r#"
        while (false) {
          continue nonexistent;
        }
        "#;
    let string = forward(&mut context, src);
    assert_eq!(
        string,
        "Uncaught \"SyntaxError\": \"Cannot use the undeclared label 'nonexistent'\""
    );
}

#[test]
fn test_invalid_continue() {
    let mut context = Context::default();
    let string = forward(&mut context, r"continue;");
    assert_eq!(
        string,
        "Uncaught \"SyntaxError\": \"continue must be inside loop\""
    );
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
fn invalid_unary_access() {
    check_output(&[
        TestAction::TestStartsWith("++[];", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("[]++;", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("--[];", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("[]--;", "Uncaught \"SyntaxError\": "),
    ]);
}

#[test]
fn typeof_string() {
    let typeof_string = r#"
        const a = String();
        typeof a;
    "#;
    assert_eq!(&exec(typeof_string), "\"string\"");
}

#[test]
fn typeof_int() {
    let typeof_int = r#"
        let a = 5;
        typeof a;
    "#;
    assert_eq!(&exec(typeof_int), "\"number\"");
}

#[test]
fn typeof_rational() {
    let typeof_rational = r#"
        let a = 0.5;
        typeof a;
    "#;
    assert_eq!(&exec(typeof_rational), "\"number\"");
}

#[test]
fn typeof_undefined() {
    let typeof_undefined = r#"
        let a = undefined;
        typeof a;
    "#;
    assert_eq!(&exec(typeof_undefined), "\"undefined\"");
}

#[test]
fn typeof_undefined_directly() {
    let typeof_undefined = r#"
        typeof undefined;
    "#;
    assert_eq!(&exec(typeof_undefined), "\"undefined\"");
}

#[test]
fn typeof_boolean() {
    let typeof_boolean = r#"
        let a = true;
        typeof a;
    "#;
    assert_eq!(&exec(typeof_boolean), "\"boolean\"");
}

#[test]
fn typeof_null() {
    let typeof_null = r#"
        let a = null;
        typeof a;
    "#;
    assert_eq!(&exec(typeof_null), "\"object\"");
}

#[test]
fn typeof_object() {
    let typeof_object = r#"
        let a = {};
        typeof a;
    "#;
    assert_eq!(&exec(typeof_object), "\"object\"");
}

#[test]
fn typeof_symbol() {
    let typeof_symbol = r#"
        let a = Symbol();
        typeof a;
    "#;
    assert_eq!(&exec(typeof_symbol), "\"symbol\"");
}

#[test]
fn typeof_function() {
    let typeof_function = r#"
        let a = function(){};
        typeof a;
    "#;
    assert_eq!(&exec(typeof_function), "\"function\"");
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
    assert_eq!(&exec(void_invocation), "\"42undefined\"");
}

#[test]
fn unary_delete() {
    let delete_var = r#"
        let a = 5;
        const b = delete a + '';
        a + b
    "#;
    assert_eq!(&exec(delete_var), "\"5false\"");

    let delete_prop = r#"
        const a = { b: 5 };
        const c = delete a.b + '';
        a.b + c
    "#;
    assert_eq!(&exec(delete_prop), "\"undefinedtrue\"");

    let delete_not_existing_prop = r#"
        const a = { b: 5 };
        const c = delete a.c + '';
        a.b + c
    "#;
    assert_eq!(&exec(delete_not_existing_prop), "\"5true\"");

    let delete_field = r#"
        const a = { b: 5 };
        const c = delete a['b'] + '';
        a.b + c
    "#;
    assert_eq!(&exec(delete_field), "\"undefinedtrue\"");

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
    use crate::forward_val;

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
    fn symbol_in_object() {
        let sym_in_object = r#"
            var sym = Symbol('hi');
            var o = {};
            o[sym] = 'hello';
            sym in o
        "#;
        assert_eq!(&exec(sym_in_object), "true");
    }

    #[test]
    fn should_type_error_when_rhs_not_object() {
        let scenario = r#"
            var x = false;
            try {
                'fail' in undefined
            } catch(e) {
                x = true;
            }
        "#;

        check_output(&[
            TestAction::Execute(scenario),
            TestAction::TestEq("x", "true"),
        ]);
    }

    #[test]
    fn should_set_this_value() {
        let scenario = r#"
        function Foo() {
            this.a = "a";
            this.b = "b";
          }

          var bar = new Foo();
        "#;
        check_output(&[
            TestAction::Execute(scenario),
            TestAction::TestEq("bar.a", "\"a\""),
            TestAction::TestEq("bar.b", "\"b\""),
        ]);
    }

    #[test]
    fn should_type_error_when_new_is_not_constructor() {
        let scenario = r#"
            const a = "";
            new a();
        "#;

        check_output(&[TestAction::TestEq(
            scenario,
            "Uncaught \"TypeError\": \"not a constructor\"",
        )]);
    }

    #[test]
    fn new_instance_should_point_to_prototype() {
        // A new instance should point to a prototype object created with the constructor function
        let mut context = Context::default();

        let scenario = r#"
            function Foo() {}
            var bar = new Foo();
        "#;
        forward(&mut context, scenario);
        let bar_val = forward_val(&mut context, "bar").unwrap();
        let bar_obj = bar_val.as_object().unwrap();
        let foo_val = forward_val(&mut context, "Foo").unwrap();
        assert_eq!(
            *bar_obj.prototype(),
            foo_val.as_object().and_then(|obj| obj
                .get("prototype", &mut context)
                .unwrap()
                .as_object()
                .cloned())
        );
    }
}

#[test]
fn var_decl_hoisting_simple() {
    let scenario = r#"
        x = 5;

        var x;
        x;
    "#;
    assert_eq!(&exec(scenario), "5");
}

#[test]
fn var_decl_hoisting_with_initialization() {
    let scenario = r#"
        x = 5;

        var x = 10;
        x;
    "#;
    assert_eq!(&exec(scenario), "10");
}

#[test]
#[ignore]
fn var_decl_hoisting_2_variables_hoisting() {
    let scenario = r#"
        x = y;

        var x = 10;
        var y = 5;

        x;
    "#;
    assert_eq!(&exec(scenario), "10");
}

#[test]
#[ignore]
fn var_decl_hoisting_2_variables_hoisting_2() {
    let scenario = r#"
        var x = y;

        var y = 5;
        x;
    "#;
    assert_eq!(&exec(scenario), "undefined");
}

#[test]
#[ignore]
fn var_decl_hoisting_2_variables_hoisting_3() {
    let scenario = r#"
        let y = x;
        x = 5;

        var x = 10;
        y;
    "#;
    assert_eq!(&exec(scenario), "undefined");
}

#[test]
fn function_decl_hoisting() {
    let scenario = r#"
        let a = hello();
        function hello() { return 5 }

        a;
    "#;
    assert_eq!(&exec(scenario), "5");

    let scenario = r#"
        x = hello();

        function hello() {return 5}
        var x;
        x;
    "#;
    assert_eq!(&exec(scenario), "5");

    let scenario = r#"
        hello = function() { return 5 }
        x = hello();

        x;
    "#;
    assert_eq!(&exec(scenario), "5");

    let scenario = r#"
        let x = b();

        function a() {return 5}
        function b() {return a()}

        x;
    "#;
    assert_eq!(&exec(scenario), "5");

    let scenario = r#"
        let x = b();

        function b() {return a()}
        function a() {return 5}

        x;
    "#;
    assert_eq!(&exec(scenario), "5");
}

#[test]
fn to_bigint() {
    let mut context = Context::default();

    assert!(JsValue::null().to_bigint(&mut context).is_err());
    assert!(JsValue::undefined().to_bigint(&mut context).is_err());
    assert!(JsValue::new(55).to_bigint(&mut context).is_err());
    assert!(JsValue::new(10.0).to_bigint(&mut context).is_err());
    assert!(JsValue::new("100").to_bigint(&mut context).is_ok());
}

#[test]
fn to_index() {
    let mut context = Context::default();

    assert_eq!(JsValue::undefined().to_index(&mut context).unwrap(), 0);
    assert!(JsValue::new(-1).to_index(&mut context).is_err());
}

#[test]
fn to_integer_or_infinity() {
    let mut context = Context::default();

    assert_eq!(
        JsValue::nan().to_integer_or_infinity(&mut context).unwrap(),
        0
    );
    assert_eq!(
        JsValue::new(f64::NEG_INFINITY)
            .to_integer_or_infinity(&mut context)
            .unwrap(),
        IntegerOrInfinity::NegativeInfinity
    );
    assert_eq!(
        JsValue::new(f64::INFINITY)
            .to_integer_or_infinity(&mut context)
            .unwrap(),
        IntegerOrInfinity::PositiveInfinity
    );
    assert_eq!(
        JsValue::new(0.0)
            .to_integer_or_infinity(&mut context)
            .unwrap(),
        0
    );
    assert_eq!(
        JsValue::new(-0.0)
            .to_integer_or_infinity(&mut context)
            .unwrap(),
        0
    );
    assert_eq!(
        JsValue::new(20.9)
            .to_integer_or_infinity(&mut context)
            .unwrap(),
        20
    );
    assert_eq!(
        JsValue::new(-20.9)
            .to_integer_or_infinity(&mut context)
            .unwrap(),
        -20
    );
}

#[test]
fn to_length() {
    let mut context = Context::default();

    assert_eq!(JsValue::new(f64::NAN).to_length(&mut context).unwrap(), 0);
    assert_eq!(
        JsValue::new(f64::NEG_INFINITY)
            .to_length(&mut context)
            .unwrap(),
        0
    );
    assert_eq!(
        JsValue::new(f64::INFINITY).to_length(&mut context).unwrap(),
        Number::MAX_SAFE_INTEGER as usize
    );
    assert_eq!(JsValue::new(0.0).to_length(&mut context).unwrap(), 0);
    assert_eq!(JsValue::new(-0.0).to_length(&mut context).unwrap(), 0);
    assert_eq!(JsValue::new(20.9).to_length(&mut context).unwrap(), 20);
    assert_eq!(JsValue::new(-20.9).to_length(&mut context).unwrap(), 0);
    assert_eq!(
        JsValue::new(100000000000.0)
            .to_length(&mut context)
            .unwrap() as u64,
        100000000000
    );
    assert_eq!(
        JsValue::new(4010101101.0).to_length(&mut context).unwrap(),
        4010101101
    );
}

#[test]
fn to_int32() {
    let mut context = Context::default();

    macro_rules! check_to_int32 {
        ($from:expr => $to:expr) => {
            assert_eq!(JsValue::new($from).to_i32(&mut context).unwrap(), $to);
        };
    }

    check_to_int32!(f64::NAN => 0);
    check_to_int32!(f64::NEG_INFINITY => 0);
    check_to_int32!(f64::INFINITY => 0);
    check_to_int32!(0 => 0);
    check_to_int32!(-0.0 => 0);

    check_to_int32!(20.9 => 20);
    check_to_int32!(-20.9 => -20);

    check_to_int32!(Number::MIN_VALUE => 0);
    check_to_int32!(-Number::MIN_VALUE => 0);
    check_to_int32!(0.1 => 0);
    check_to_int32!(-0.1 => 0);
    check_to_int32!(1 => 1);
    check_to_int32!(1.1 => 1);
    check_to_int32!(-1 => -1);
    check_to_int32!(0.6 => 0);
    check_to_int32!(1.6 => 1);
    check_to_int32!(-0.6 => 0);
    check_to_int32!(-1.6 => -1);

    check_to_int32!(2147483647.0 => 2147483647);
    check_to_int32!(2147483648.0 => -2147483648);
    check_to_int32!(2147483649.0 => -2147483647);

    check_to_int32!(4294967295.0 => -1);
    check_to_int32!(4294967296.0 => 0);
    check_to_int32!(4294967297.0 => 1);

    check_to_int32!(-2147483647.0 => -2147483647);
    check_to_int32!(-2147483648.0 => -2147483648);
    check_to_int32!(-2147483649.0 => 2147483647);

    check_to_int32!(-4294967295.0 => 1);
    check_to_int32!(-4294967296.0 => 0);
    check_to_int32!(-4294967297.0 => -1);

    check_to_int32!(2147483648.25 => -2147483648);
    check_to_int32!(2147483648.5 => -2147483648);
    check_to_int32!(2147483648.75 => -2147483648);
    check_to_int32!(4294967295.25 => -1);
    check_to_int32!(4294967295.5 => -1);
    check_to_int32!(4294967295.75 => -1);
    check_to_int32!(3000000000.25 => -1294967296);
    check_to_int32!(3000000000.5 => -1294967296);
    check_to_int32!(3000000000.75 => -1294967296);

    check_to_int32!(-2147483648.25 => -2147483648);
    check_to_int32!(-2147483648.5 => -2147483648);
    check_to_int32!(-2147483648.75 => -2147483648);
    check_to_int32!(-4294967295.25 => 1);
    check_to_int32!(-4294967295.5 => 1);
    check_to_int32!(-4294967295.75 => 1);
    check_to_int32!(-3000000000.25 => 1294967296);
    check_to_int32!(-3000000000.5 => 1294967296);
    check_to_int32!(-3000000000.75 => 1294967296);

    let base = 2f64.powf(64.0);
    check_to_int32!(base + 0.0 => 0);
    check_to_int32!(base + 1117.0 => 0);
    check_to_int32!(base + 2234.0 => 4096);
    check_to_int32!(base + 3351.0 => 4096);
    check_to_int32!(base + 4468.0 => 4096);
    check_to_int32!(base + 5585.0 => 4096);
    check_to_int32!(base + 6702.0 => 8192);
    check_to_int32!(base + 7819.0 => 8192);
    check_to_int32!(base + 8936.0 => 8192);
    check_to_int32!(base + 10053.0 => 8192);
    check_to_int32!(base + 11170.0 => 12288);
    check_to_int32!(base + 12287.0 => 12288);
    check_to_int32!(base + 13404.0 => 12288);
    check_to_int32!(base + 14521.0 => 16384);
    check_to_int32!(base + 15638.0 => 16384);
    check_to_int32!(base + 16755.0 => 16384);
    check_to_int32!(base + 17872.0 => 16384);
    check_to_int32!(base + 18989.0 => 20480);
    check_to_int32!(base + 20106.0 => 20480);
    check_to_int32!(base + 21223.0 => 20480);
    check_to_int32!(base + 22340.0 => 20480);
    check_to_int32!(base + 23457.0 => 24576);
    check_to_int32!(base + 24574.0 => 24576);
    check_to_int32!(base + 25691.0 => 24576);
    check_to_int32!(base + 26808.0 => 28672);
    check_to_int32!(base + 27925.0 => 28672);
    check_to_int32!(base + 29042.0 => 28672);
    check_to_int32!(base + 30159.0 => 28672);
    check_to_int32!(base + 31276.0 => 32768);

    // bignum is (2^53 - 1) * 2^31 - highest number with bit 31 set.
    let bignum = 2f64.powf(84.0) - 2f64.powf(31.0);
    check_to_int32!(bignum => -2147483648);
    check_to_int32!(-bignum => -2147483648);
    check_to_int32!(2.0 * bignum => 0);
    check_to_int32!(-(2.0 * bignum) => 0);
    check_to_int32!(bignum - 2f64.powf(31.0) => 0);
    check_to_int32!(-(bignum - 2f64.powf(31.0)) => 0);

    // max_fraction is largest number below 1.
    let max_fraction = 1.0 - 2f64.powf(-53.0);
    check_to_int32!(max_fraction => 0);
    check_to_int32!(-max_fraction => 0);
}

#[test]
fn to_string() {
    let mut context = Context::default();

    assert_eq!(JsValue::null().to_string(&mut context).unwrap(), "null");
    assert_eq!(
        JsValue::undefined().to_string(&mut context).unwrap(),
        "undefined"
    );
    assert_eq!(JsValue::new(55).to_string(&mut context).unwrap(), "55");
    assert_eq!(JsValue::new(55.0).to_string(&mut context).unwrap(), "55");
    assert_eq!(
        JsValue::new("hello").to_string(&mut context).unwrap(),
        "hello"
    );
}

#[test]
fn calling_function_with_unspecified_arguments() {
    let mut context = Context::default();
    let scenario = r#"
        function test(a, b) {
            return b;
        }

        test(10)
    "#;

    assert_eq!(forward(&mut context, scenario), "undefined");
}

#[test]
fn to_object() {
    let mut context = Context::default();

    assert!(JsValue::undefined()
        .to_object(&mut context)
        .unwrap_err()
        .is_object());
    assert!(JsValue::null()
        .to_object(&mut context)
        .unwrap_err()
        .is_object());
}

#[test]
fn check_this_binding_in_object_literal() {
    let mut context = Context::default();
    let init = r#"
        var foo = {
            a: 3,
            bar: function () { return this.a + 5 }
        };

        foo.bar()
        "#;

    assert_eq!(forward(&mut context, init), "8");
}

#[test]
fn array_creation_benchmark() {
    let mut context = Context::default();
    let init = r#"
        (function(){
            let testArr = [];
            for (let a = 0; a <= 500; a++) {
                testArr[a] = ('p' + a);
            }

            return testArr;
        })();
        "#;

    assert_eq!(forward(&mut context, init), "[ \"p0\", \"p1\", \"p2\", \"p3\", \"p4\", \"p5\", \"p6\", \"p7\", \"p8\", \"p9\", \"p10\", \"p11\", \"p12\", \"p13\", \"p14\", \"p15\", \"p16\", \"p17\", \"p18\", \"p19\", \"p20\", \"p21\", \"p22\", \"p23\", \"p24\", \"p25\", \"p26\", \"p27\", \"p28\", \"p29\", \"p30\", \"p31\", \"p32\", \"p33\", \"p34\", \"p35\", \"p36\", \"p37\", \"p38\", \"p39\", \"p40\", \"p41\", \"p42\", \"p43\", \"p44\", \"p45\", \"p46\", \"p47\", \"p48\", \"p49\", \"p50\", \"p51\", \"p52\", \"p53\", \"p54\", \"p55\", \"p56\", \"p57\", \"p58\", \"p59\", \"p60\", \"p61\", \"p62\", \"p63\", \"p64\", \"p65\", \"p66\", \"p67\", \"p68\", \"p69\", \"p70\", \"p71\", \"p72\", \"p73\", \"p74\", \"p75\", \"p76\", \"p77\", \"p78\", \"p79\", \"p80\", \"p81\", \"p82\", \"p83\", \"p84\", \"p85\", \"p86\", \"p87\", \"p88\", \"p89\", \"p90\", \"p91\", \"p92\", \"p93\", \"p94\", \"p95\", \"p96\", \"p97\", \"p98\", \"p99\", \"p100\", \"p101\", \"p102\", \"p103\", \"p104\", \"p105\", \"p106\", \"p107\", \"p108\", \"p109\", \"p110\", \"p111\", \"p112\", \"p113\", \"p114\", \"p115\", \"p116\", \"p117\", \"p118\", \"p119\", \"p120\", \"p121\", \"p122\", \"p123\", \"p124\", \"p125\", \"p126\", \"p127\", \"p128\", \"p129\", \"p130\", \"p131\", \"p132\", \"p133\", \"p134\", \"p135\", \"p136\", \"p137\", \"p138\", \"p139\", \"p140\", \"p141\", \"p142\", \"p143\", \"p144\", \"p145\", \"p146\", \"p147\", \"p148\", \"p149\", \"p150\", \"p151\", \"p152\", \"p153\", \"p154\", \"p155\", \"p156\", \"p157\", \"p158\", \"p159\", \"p160\", \"p161\", \"p162\", \"p163\", \"p164\", \"p165\", \"p166\", \"p167\", \"p168\", \"p169\", \"p170\", \"p171\", \"p172\", \"p173\", \"p174\", \"p175\", \"p176\", \"p177\", \"p178\", \"p179\", \"p180\", \"p181\", \"p182\", \"p183\", \"p184\", \"p185\", \"p186\", \"p187\", \"p188\", \"p189\", \"p190\", \"p191\", \"p192\", \"p193\", \"p194\", \"p195\", \"p196\", \"p197\", \"p198\", \"p199\", \"p200\", \"p201\", \"p202\", \"p203\", \"p204\", \"p205\", \"p206\", \"p207\", \"p208\", \"p209\", \"p210\", \"p211\", \"p212\", \"p213\", \"p214\", \"p215\", \"p216\", \"p217\", \"p218\", \"p219\", \"p220\", \"p221\", \"p222\", \"p223\", \"p224\", \"p225\", \"p226\", \"p227\", \"p228\", \"p229\", \"p230\", \"p231\", \"p232\", \"p233\", \"p234\", \"p235\", \"p236\", \"p237\", \"p238\", \"p239\", \"p240\", \"p241\", \"p242\", \"p243\", \"p244\", \"p245\", \"p246\", \"p247\", \"p248\", \"p249\", \"p250\", \"p251\", \"p252\", \"p253\", \"p254\", \"p255\", \"p256\", \"p257\", \"p258\", \"p259\", \"p260\", \"p261\", \"p262\", \"p263\", \"p264\", \"p265\", \"p266\", \"p267\", \"p268\", \"p269\", \"p270\", \"p271\", \"p272\", \"p273\", \"p274\", \"p275\", \"p276\", \"p277\", \"p278\", \"p279\", \"p280\", \"p281\", \"p282\", \"p283\", \"p284\", \"p285\", \"p286\", \"p287\", \"p288\", \"p289\", \"p290\", \"p291\", \"p292\", \"p293\", \"p294\", \"p295\", \"p296\", \"p297\", \"p298\", \"p299\", \"p300\", \"p301\", \"p302\", \"p303\", \"p304\", \"p305\", \"p306\", \"p307\", \"p308\", \"p309\", \"p310\", \"p311\", \"p312\", \"p313\", \"p314\", \"p315\", \"p316\", \"p317\", \"p318\", \"p319\", \"p320\", \"p321\", \"p322\", \"p323\", \"p324\", \"p325\", \"p326\", \"p327\", \"p328\", \"p329\", \"p330\", \"p331\", \"p332\", \"p333\", \"p334\", \"p335\", \"p336\", \"p337\", \"p338\", \"p339\", \"p340\", \"p341\", \"p342\", \"p343\", \"p344\", \"p345\", \"p346\", \"p347\", \"p348\", \"p349\", \"p350\", \"p351\", \"p352\", \"p353\", \"p354\", \"p355\", \"p356\", \"p357\", \"p358\", \"p359\", \"p360\", \"p361\", \"p362\", \"p363\", \"p364\", \"p365\", \"p366\", \"p367\", \"p368\", \"p369\", \"p370\", \"p371\", \"p372\", \"p373\", \"p374\", \"p375\", \"p376\", \"p377\", \"p378\", \"p379\", \"p380\", \"p381\", \"p382\", \"p383\", \"p384\", \"p385\", \"p386\", \"p387\", \"p388\", \"p389\", \"p390\", \"p391\", \"p392\", \"p393\", \"p394\", \"p395\", \"p396\", \"p397\", \"p398\", \"p399\", \"p400\", \"p401\", \"p402\", \"p403\", \"p404\", \"p405\", \"p406\", \"p407\", \"p408\", \"p409\", \"p410\", \"p411\", \"p412\", \"p413\", \"p414\", \"p415\", \"p416\", \"p417\", \"p418\", \"p419\", \"p420\", \"p421\", \"p422\", \"p423\", \"p424\", \"p425\", \"p426\", \"p427\", \"p428\", \"p429\", \"p430\", \"p431\", \"p432\", \"p433\", \"p434\", \"p435\", \"p436\", \"p437\", \"p438\", \"p439\", \"p440\", \"p441\", \"p442\", \"p443\", \"p444\", \"p445\", \"p446\", \"p447\", \"p448\", \"p449\", \"p450\", \"p451\", \"p452\", \"p453\", \"p454\", \"p455\", \"p456\", \"p457\", \"p458\", \"p459\", \"p460\", \"p461\", \"p462\", \"p463\", \"p464\", \"p465\", \"p466\", \"p467\", \"p468\", \"p469\", \"p470\", \"p471\", \"p472\", \"p473\", \"p474\", \"p475\", \"p476\", \"p477\", \"p478\", \"p479\", \"p480\", \"p481\", \"p482\", \"p483\", \"p484\", \"p485\", \"p486\", \"p487\", \"p488\", \"p489\", \"p490\", \"p491\", \"p492\", \"p493\", \"p494\", \"p495\", \"p496\", \"p497\", \"p498\", \"p499\", \"p500\" ]");
}

#[test]
fn array_pop_benchmark() {
    let mut context = Context::default();
    let init = r#"
    (function(){
        let testArray = [83, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32,
                         45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828,
                         234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62,
                         99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67,
                         77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29,
                         2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28,
                         83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32,
                         56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93,
                         27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93,
                         17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23,
                         56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36,
                         28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32,
                         45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234,
                         23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99,
                         36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32,
                         45, 93, 17, 28, 83, 62, 99, 36, 28];

        while (testArray.length > 0) {
            testArray.pop();
        }

        return testArray;
    })();
    "#;

    assert_eq!(forward(&mut context, init), "[]");
}

#[test]
fn number_object_access_benchmark() {
    let mut context = Context::default();
    let init = r#"
    new Number(
        new Number(
            new Number(
                new Number(100).valueOf() - 10.5
            ).valueOf() + 100
        ).valueOf() * 1.6
    )
    "#;

    assert!(forward_val(&mut context, init).is_ok());
}

#[test]
fn not_a_function() {
    let init = r#"
        let a = {};
        let b = true;
        "#;
    let scenario1 = r#"
        try {
            a();
        } catch(e) {
            e.toString()
        }
    "#;
    let scenario2 = r#"
        try {
            a.a();
        } catch(e) {
            e.toString()
        }
    "#;
    let scenario3 = r#"
        try {
            b();
        } catch(e) {
            e.toString()
        }
    "#;

    check_output(&[
        TestAction::Execute(init),
        TestAction::TestEq(scenario1, "\"TypeError: not a callable function\""),
        TestAction::TestEq(scenario2, "\"TypeError: not a callable function\""),
        TestAction::TestEq(scenario3, "\"TypeError: not a callable function\""),
    ]);
}

#[test]
fn comma_operator() {
    let scenario = r#"
        var a, b;
        b = 10;
        a = (b++, b);
        a
    "#;
    assert_eq!(&exec(scenario), "11");

    let scenario = r#"
        var a, b;
        b = 10;
        a = (b += 5, b /= 3, b - 3);
        a
    "#;
    assert_eq!(&exec(scenario), "2");
}

#[test]
fn assignment_to_non_assignable() {
    // Relates to the behaviour described at
    // https://tc39.es/ecma262/#sec-assignment-operators-static-semantics-early-errors
    let mut context = Context::default();

    // Tests all assignment operators as per [spec] and [mdn]
    //
    // [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Assignment
    // [spec]: https://tc39.es/ecma262/#prod-AssignmentOperator
    let test_cases = [
        "3 -= 5", "3 *= 5", "3 /= 5", "3 %= 5", "3 &= 5", "3 ^= 5", "3 |= 5", "3 += 5", "3 = 5",
    ];

    for case in &test_cases {
        let string = forward(&mut context, case);

        assert!(string.starts_with("Uncaught \"SyntaxError\": "));
        assert!(string.contains("1:3"));
    }
}

#[test]
fn assignment_to_non_assignable_ctd() {
    check_output(&[
        TestAction::TestStartsWith("(()=>{})() -= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() *= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() /= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() %= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() &= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() ^= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() |= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() += 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() = 5", "Uncaught \"SyntaxError\": "),
    ]);
}

#[test]
fn multicharacter_assignment_to_non_assignable() {
    // Relates to the behaviour described at
    // https://tc39.es/ecma262/#sec-assignment-operators-static-semantics-early-errors
    let mut context = Context::default();

    let test_cases = ["3 **= 5", "3 <<= 5", "3 >>= 5"];

    for case in &test_cases {
        let string = dbg!(forward(&mut context, case));

        assert!(string.starts_with("Uncaught \"SyntaxError\": "));
        assert!(string.contains("1:3"));
    }
}

#[test]
fn multicharacter_assignment_to_non_assignable_ctd() {
    check_output(&[
        TestAction::TestStartsWith("(()=>{})() **= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() <<= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() >>= 5", "Uncaught \"SyntaxError\": "),
    ]);
}

#[test]
fn multicharacter_bitwise_assignment_to_non_assignable() {
    let mut context = Context::default();

    // Disabled - awaiting implementation.
    let test_cases = ["3 >>>= 5", "3 &&= 5", "3 ||= 5", "3 ??= 5"];

    for case in &test_cases {
        let string = dbg!(forward(&mut context, case));

        assert!(string.starts_with("Uncaught \"SyntaxError\": "));
        assert!(string.contains("1:3"));
    }
}

#[test]
fn multicharacter_bitwise_assignment_to_non_assignable_ctd() {
    check_output(&[
        TestAction::TestStartsWith("(()=>{})() >>>= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() &&= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() ||= 5", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("(()=>{})() ??= 5", "Uncaught \"SyntaxError\": "),
    ]);
}

#[test]
fn assign_to_array_decl() {
    check_output(&[
        TestAction::TestStartsWith("[1] = [2]", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("[3, 5] = [7, 8]", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("[6, 8] = [2]", "Uncaught \"SyntaxError\": "),
        TestAction::TestStartsWith("[6] = [2, 9]", "Uncaught \"SyntaxError\": "),
    ]);
}

#[test]
fn assign_to_object_decl() {
    const ERR_MSG: &str =
        "Uncaught \"SyntaxError\": \"unexpected token '=', primary expression at line 1, col 8\"";

    let mut context = Context::default();

    assert_eq!(forward(&mut context, "{a: 3} = {a: 5};"), ERR_MSG);
}

#[test]
fn multiline_str_concat() {
    let scenario = r#"
        let a = 'hello ' +
                'world';

        a"#;
    assert_eq!(&exec(scenario), "\"hello world\"");
}

#[test]
fn test_result_of_empty_block() {
    let scenario = "{}";
    assert_eq!(&exec(scenario), "undefined");
}

#[test]
fn test_undefined_constant() {
    let scenario = "undefined";
    assert_eq!(&exec(scenario), "undefined");
}

#[test]
fn test_undefined_type() {
    let scenario = "typeof undefined";
    assert_eq!(&exec(scenario), "\"undefined\"");
}

#[test]
fn test_conditional_op() {
    let scenario = "1 === 2 ? 'a' : 'b'";
    assert_eq!(&exec(scenario), "\"b\"");
}

#[test]
fn test_identifier_op() {
    let scenario = "break = 1";
    assert_eq!(&exec(scenario), "\"SyntaxError\": \"expected token \'identifier\', got \'=\' in binding identifier at line 1, col 7\"");
}

#[test]
fn test_strict_mode_octal() {
    // Checks as per https://tc39.es/ecma262/#sec-literals-numeric-literals that 0 prefix
    // octal number literal syntax is a syntax error in strict mode.

    let scenario = r#"
    'use strict';
    var n = 023;
    "#;

    check_output(&[TestAction::TestStartsWith(
        scenario,
        "Uncaught \"SyntaxError\": ",
    )]);
}

#[test]
fn test_strict_mode_with() {
    // Checks as per https://tc39.es/ecma262/#sec-with-statement-static-semantics-early-errors
    // that a with statement is an error in strict mode code.

    let scenario = r#"
    'use strict';
    function f(x, o) {
        with (o) {
            console.log(x);
        }
    }
    "#;

    check_output(&[TestAction::TestStartsWith(
        scenario,
        "Uncaught \"SyntaxError\": ",
    )]);
}

#[test]
fn test_strict_mode_delete() {
    // Checks as per https://tc39.es/ecma262/#sec-delete-operator-static-semantics-early-errors
    // that delete on a variable name is an error in strict mode code.

    let scenario = r#"
    'use strict';
    let x = 10;
    delete x;
    "#;

    check_output(&[TestAction::TestStartsWith(
        scenario,
        "Uncaught \"SyntaxError\": ",
    )]);
}

#[test]
fn test_strict_mode_reserved_name() {
    // Checks that usage of a reserved keyword for an identifier name is
    // an error in strict mode code as per https://tc39.es/ecma262/#sec-strict-mode-of-ecmascript.

    let test_cases = [
        "var implements = 10;",
        "var interface = 10;",
        "var package = 10;",
        "var private = 10;",
        "var protected = 10;",
        "var public = 10;",
        "var static = 10;",
        "var eval = 10;",
        "var arguments = 10;",
        "var let = 10;",
        "var yield = 10;",
    ];

    for case in &test_cases {
        let mut context = Context::default();
        let scenario = format!("'use strict'; \n {case}");

        let string = dbg!(forward(&mut context, &scenario));

        assert!(string.starts_with("Uncaught \"SyntaxError\": "));
    }
}

#[test]
fn test_strict_mode_func_decl_in_block() {
    // Checks that a function declaration in a block is an error in
    // strict mode code as per https://tc39.es/ecma262/#early-error.

    let scenario = r#"
    'use strict';
    let a = 4;
    let b = 5;
    if (a < b) { function f() {} }
    "#;

    check_output(&[TestAction::TestStartsWith(
        scenario,
        "Uncaught \"SyntaxError\": ",
    )]);
}

#[test]
fn test_strict_mode_dup_func_parameters() {
    // Checks that a function cannot contain duplicate parameter
    // names in strict mode code as per https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors.

    let scenario = r#"
    'use strict';
    function f(a, b, b) {}
    "#;

    check_output(&[TestAction::TestStartsWith(
        scenario,
        "Uncaught \"SyntaxError\": ",
    )]);
}

#[test]
fn test_empty_statement() {
    let src = r#"
        ;;;let a = 10;;
        if(a) ;
        a
    "#;
    assert_eq!(&exec(src), "10");
}
