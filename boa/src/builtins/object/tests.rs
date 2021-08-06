use crate::{forward, Context, JsValue};

#[test]
fn object_create_with_regular_object() {
    let mut context = Context::new();

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
    let mut context = Context::new();

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
    let mut context = Context::new();

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
    let mut context = Context::new();

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
    let mut context = Context::new();

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
    let mut context = Context::new();
    let init = r#"
        let x = { someProp: 1, undefinedProp: undefined, nullProp: null };
    "#;

    eprintln!("{}", forward(&mut context, init));
    assert_eq!(
        forward(&mut context, "x.hasOwnProperty('someProp')"),
        "true"
    );
    assert_eq!(
        forward(&mut context, "x.hasOwnProperty('undefinedProp')"),
        "true"
    );
    assert_eq!(
        forward(&mut context, "x.hasOwnProperty('nullProp')"),
        "true"
    );
    assert_eq!(
        forward(&mut context, "x.hasOwnProperty('hasOwnProperty')"),
        "false"
    );
}

#[test]
fn object_property_is_enumerable() {
    let mut context = Context::new();
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
    )
}

#[test]
fn object_to_string() {
    let mut context = Context::new();
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
    let mut context = Context::new();

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
    let mut context = Context::new();
    let code = r#"
        let obj = {a: 2};
        Object.getOwnPropertyDescriptor(obj)
    "#;
    assert_eq!(context.eval(code).unwrap(), JsValue::undefined());
}

#[test]
fn get_own_property_descriptor() {
    let mut context = Context::new();
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
    let mut context = Context::new();
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
    let mut context = Context::new();

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
    let mut context = Context::new();

    let init = r#"
        Object.prototype.isPrototypeOf(String.prototype)
    "#;

    assert_eq!(context.eval(init).unwrap(), JsValue::new(true));
}
