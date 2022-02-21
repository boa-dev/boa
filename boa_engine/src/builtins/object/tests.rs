use crate::{check_output, forward, Context, JsValue, TestAction};

#[test]
fn object_create_with_regular_object() {
    let mut context = Context::default();

    let init = r#"
        const foo = { a: 5 };
        const bar = Object.create(foo);
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "bar.a"), "5");
    assert_eq!(forward(&mut context, "Object.create.length"), "2");
}

#[test]
fn object_create_with_undefined() {
    let mut context = Context::default();

    let init = r#"
        try {
            const bar = Object.create();
        } catch (err) {
            err.toString()
        }
        "#;

    let result = forward(&mut context, init);
    assert_eq!(
        result,
        "\"TypeError: Object prototype may only be an Object or null: undefined\""
    );
}

#[test]
fn object_create_with_number() {
    let mut context = Context::default();

    let init = r#"
        try {
            const bar = Object.create(5);
        } catch (err) {
            err.toString()
        }
        "#;

    let result = forward(&mut context, init);
    assert_eq!(
        result,
        "\"TypeError: Object prototype may only be an Object or null: 5\""
    );
}

#[test]
#[ignore]
// TODO: to test on __proto__ somehow. __proto__ getter is not working as expected currently
fn object_create_with_function() {
    let mut context = Context::default();

    let init = r#"
        const x = function (){};
        const bar = Object.create(5);
        bar.__proto__
        "#;

    let result = forward(&mut context, init);
    assert_eq!(result, "...something on __proto__...");
}

#[test]
fn object_is() {
    let mut context = Context::default();

    let init = r#"
        var foo = { a: 1};
        var bar = { a: 1};
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "Object.is('foo', 'foo')"), "true");
    assert_eq!(forward(&mut context, "Object.is('foo', 'bar')"), "false");
    assert_eq!(forward(&mut context, "Object.is([], [])"), "false");
    assert_eq!(forward(&mut context, "Object.is(foo, foo)"), "true");
    assert_eq!(forward(&mut context, "Object.is(foo, bar)"), "false");
    assert_eq!(forward(&mut context, "Object.is(null, null)"), "true");
    assert_eq!(forward(&mut context, "Object.is(0, -0)"), "false");
    assert_eq!(forward(&mut context, "Object.is(-0, -0)"), "true");
    assert_eq!(forward(&mut context, "Object.is(NaN, 0/0)"), "true");
    assert_eq!(forward(&mut context, "Object.is()"), "true");
    assert_eq!(forward(&mut context, "Object.is(undefined)"), "true");
    assert!(context.global_object().is_global());
}

#[test]
fn object_has_own_property() {
    let scenario = r#"
        let symA = Symbol('a');
        let symB = Symbol('b');

        let x = {
            undefinedProp: undefined,
            nullProp: null,
            someProp: 1,
            [symA]: 2,
            100: 3,
        };
    "#;

    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("x.hasOwnProperty('hasOwnProperty')", "false"),
        TestAction::TestEq("x.hasOwnProperty('undefinedProp')", "true"),
        TestAction::TestEq("x.hasOwnProperty('nullProp')", "true"),
        TestAction::TestEq("x.hasOwnProperty('someProp')", "true"),
        TestAction::TestEq("x.hasOwnProperty(symB)", "false"),
        TestAction::TestEq("x.hasOwnProperty(symA)", "true"),
        TestAction::TestEq("x.hasOwnProperty(1000)", "false"),
        TestAction::TestEq("x.hasOwnProperty(100)", "true"),
    ]);
}

#[test]
fn object_has_own() {
    let scenario = r#"
        let symA = Symbol('a');
        let symB = Symbol('b');

        let x = {
            undefinedProp: undefined,
            nullProp: null,
            someProp: 1,
            [symA]: 2,
            100: 3,
        };
    "#;

    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("Object.hasOwn(x, 'hasOwnProperty')", "false"),
        TestAction::TestEq("Object.hasOwn(x, 'undefinedProp')", "true"),
        TestAction::TestEq("Object.hasOwn(x, 'nullProp')", "true"),
        TestAction::TestEq("Object.hasOwn(x, 'someProp')", "true"),
        TestAction::TestEq("Object.hasOwn(x, symB)", "false"),
        TestAction::TestEq("Object.hasOwn(x, symA)", "true"),
        TestAction::TestEq("Object.hasOwn(x, 1000)", "false"),
        TestAction::TestEq("Object.hasOwn(x, 100)", "true"),
    ]);
}

#[test]
fn object_property_is_enumerable() {
    let mut context = Context::default();
    let init = r#"
        let x = { enumerableProp: 'yes' };
    "#;
    eprintln!("{}", forward(&mut context, init));
    assert_eq!(
        forward(&mut context, r#"x.propertyIsEnumerable('enumerableProp')"#),
        "true"
    );
    assert_eq!(
        forward(&mut context, r#"x.propertyIsEnumerable('hasOwnProperty')"#),
        "false"
    );
    assert_eq!(
        forward(&mut context, r#"x.propertyIsEnumerable('not_here')"#),
        "false",
    );
    assert_eq!(
        forward(&mut context, r#"x.propertyIsEnumerable()"#),
        "false",
    );
}

#[test]
fn object_to_string() {
    let mut context = Context::default();
    let init = r#"
        let u = undefined;
        let n = null;
        let a = [];
        Array.prototype.toString = Object.prototype.toString;
        let f = () => {};
        Function.prototype.toString = Object.prototype.toString;
        let e = new Error('test');
        Error.prototype.toString = Object.prototype.toString;
        let b = Boolean();
        Boolean.prototype.toString = Object.prototype.toString;
        let i = Number(42);
        Number.prototype.toString = Object.prototype.toString;
        let s = String('boa');
        String.prototype.toString = Object.prototype.toString;
        let d = new Date(Date.now());
        Date.prototype.toString = Object.prototype.toString;
        let re = /boa/;
        RegExp.prototype.toString = Object.prototype.toString;
        let o = Object();
    "#;
    eprintln!("{}", forward(&mut context, init));
    assert_eq!(
        forward(&mut context, "Object.prototype.toString.call(u)"),
        "\"[object Undefined]\""
    );
    assert_eq!(
        forward(&mut context, "Object.prototype.toString.call(n)"),
        "\"[object Null]\""
    );
    assert_eq!(forward(&mut context, "a.toString()"), "\"[object Array]\"");
    assert_eq!(
        forward(&mut context, "f.toString()"),
        "\"[object Function]\""
    );
    assert_eq!(forward(&mut context, "e.toString()"), "\"[object Error]\"");
    assert_eq!(
        forward(&mut context, "b.toString()"),
        "\"[object Boolean]\""
    );
    assert_eq!(forward(&mut context, "i.toString()"), "\"[object Number]\"");
    assert_eq!(forward(&mut context, "s.toString()"), "\"[object String]\"");
    assert_eq!(forward(&mut context, "d.toString()"), "\"[object Date]\"");
    assert_eq!(
        forward(&mut context, "re.toString()"),
        "\"[object RegExp]\""
    );
    assert_eq!(forward(&mut context, "o.toString()"), "\"[object Object]\"");
}

#[test]
fn define_symbol_property() {
    let mut context = Context::default();

    let init = r#"
        let obj = {};
        let sym = Symbol("key");
        Object.defineProperty(obj, sym, { value: "val" });
    "#;
    eprintln!("{}", forward(&mut context, init));

    assert_eq!(forward(&mut context, "obj[sym]"), "\"val\"");
}

#[test]
fn get_own_property_descriptor_1_arg_returns_undefined() {
    let mut context = Context::default();
    let code = r#"
        let obj = {a: 2};
        Object.getOwnPropertyDescriptor(obj)
    "#;
    assert_eq!(context.eval(code).unwrap(), JsValue::undefined());
}

#[test]
fn get_own_property_descriptor() {
    let mut context = Context::default();
    forward(
        &mut context,
        r#"
        let obj = {a: 2};
        let result = Object.getOwnPropertyDescriptor(obj, "a");
    "#,
    );

    assert_eq!(forward(&mut context, "result.enumerable"), "true");
    assert_eq!(forward(&mut context, "result.writable"), "true");
    assert_eq!(forward(&mut context, "result.configurable"), "true");
    assert_eq!(forward(&mut context, "result.value"), "2");
}

#[test]
fn get_own_property_descriptors() {
    let mut context = Context::default();
    forward(
        &mut context,
        r#"
        let obj = {a: 1, b: 2};
        let result = Object.getOwnPropertyDescriptors(obj);
    "#,
    );

    assert_eq!(forward(&mut context, "result.a.enumerable"), "true");
    assert_eq!(forward(&mut context, "result.a.writable"), "true");
    assert_eq!(forward(&mut context, "result.a.configurable"), "true");
    assert_eq!(forward(&mut context, "result.a.value"), "1");

    assert_eq!(forward(&mut context, "result.b.enumerable"), "true");
    assert_eq!(forward(&mut context, "result.b.writable"), "true");
    assert_eq!(forward(&mut context, "result.b.configurable"), "true");
    assert_eq!(forward(&mut context, "result.b.value"), "2");
}

#[test]
fn object_define_properties() {
    let mut context = Context::default();

    let init = r#"
        const obj = {};

        Object.defineProperties(obj, {
            p: {
                value: 42,
                writable: true
            }
        });
    "#;
    eprintln!("{}", forward(&mut context, init));

    assert_eq!(forward(&mut context, "obj.p"), "42");
}

#[test]
fn object_is_prototype_of() {
    let mut context = Context::default();

    let init = r#"
        Object.prototype.isPrototypeOf(String.prototype)
    "#;

    assert_eq!(context.eval(init).unwrap(), JsValue::new(true));
}

#[test]
fn object_get_own_property_names_invalid_args() {
    let error_message = r#"Uncaught "TypeError": "cannot convert 'null' or 'undefined' to object""#;

    check_output(&[
        TestAction::TestEq("Object.getOwnPropertyNames()", error_message),
        TestAction::TestEq("Object.getOwnPropertyNames(null)", error_message),
        TestAction::TestEq("Object.getOwnPropertyNames(undefined)", error_message),
    ]);
}

#[test]
fn object_get_own_property_names() {
    check_output(&[
        TestAction::TestEq("Object.getOwnPropertyNames(0)", "[]"),
        TestAction::TestEq("Object.getOwnPropertyNames(false)", "[]"),
        TestAction::TestEq(r#"Object.getOwnPropertyNames(Symbol("a"))"#, "[]"),
        TestAction::TestEq("Object.getOwnPropertyNames({})", "[]"),
        TestAction::TestEq("Object.getOwnPropertyNames(NaN)", "[]"),
        TestAction::TestEq(
            "Object.getOwnPropertyNames([1, 2, 3])",
            r#"[ "0", "1", "2", "length" ]"#,
        ),
        TestAction::TestEq(
            r#"Object.getOwnPropertyNames({
                "a": 1,
                "b": 2,
                [ Symbol("c") ]: 3,
                [ Symbol("d") ]: 4,
            })"#,
            r#"[ "a", "b" ]"#,
        ),
    ]);
}

#[test]
fn object_get_own_property_symbols_invalid_args() {
    let error_message = r#"Uncaught "TypeError": "cannot convert 'null' or 'undefined' to object""#;

    check_output(&[
        TestAction::TestEq("Object.getOwnPropertySymbols()", error_message),
        TestAction::TestEq("Object.getOwnPropertySymbols(null)", error_message),
        TestAction::TestEq("Object.getOwnPropertySymbols(undefined)", error_message),
    ]);
}

#[test]
fn object_get_own_property_symbols() {
    check_output(&[
        TestAction::TestEq("Object.getOwnPropertySymbols(0)", "[]"),
        TestAction::TestEq("Object.getOwnPropertySymbols(false)", "[]"),
        TestAction::TestEq(r#"Object.getOwnPropertySymbols(Symbol("a"))"#, "[]"),
        TestAction::TestEq("Object.getOwnPropertySymbols({})", "[]"),
        TestAction::TestEq("Object.getOwnPropertySymbols(NaN)", "[]"),
        TestAction::TestEq("Object.getOwnPropertySymbols([1, 2, 3])", "[]"),
        TestAction::TestEq(
            r#"
            Object.getOwnPropertySymbols({
                "a": 1,
                "b": 2,
                [ Symbol("c") ]: 3,
                [ Symbol("d") ]: 4,
            })"#,
            "[ Symbol(c), Symbol(d) ]",
        ),
    ]);
}

#[test]
fn object_from_entries_invalid_args() {
    let error_message = r#"Uncaught "TypeError": "cannot convert null or undefined to Object""#;

    check_output(&[
        TestAction::TestEq("Object.fromEntries()", error_message),
        TestAction::TestEq("Object.fromEntries(null)", error_message),
        TestAction::TestEq("Object.fromEntries(undefined)", error_message),
    ]);
}

#[test]
fn object_from_entries() {
    let scenario = r#"
        let sym = Symbol("sym");
        let map = Object.fromEntries([
            ["long key", 1],
            ["short", 2],
            [sym, 3],
            [5, 4],
        ]);
    "#;

    check_output(&[
        TestAction::Execute(scenario),
        TestAction::TestEq("map['long key']", "1"),
        TestAction::TestEq("map.short", "2"),
        TestAction::TestEq("map[sym]", "3"),
        TestAction::TestEq("map[5]", "4"),
    ]);
}
