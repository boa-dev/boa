use crate::{forward, Context};

#[test]
fn object_create_with_regular_object() {
    let mut engine = Context::new();

    let init = r#"
        const foo = { a: 5 };
        const bar = Object.create(foo);
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "bar.a"), "5");
    assert_eq!(forward(&mut engine, "Object.create.length"), "2");
}

#[test]
fn object_create_with_undefined() {
    let mut engine = Context::new();

    let init = r#"
        try {
            const bar = Object.create();
        } catch (err) {
            err.toString()
        }
        "#;

    let result = forward(&mut engine, init);
    assert_eq!(
        result,
        "\"TypeError: Object prototype may only be an Object or null: undefined\""
    );
}

#[test]
fn object_create_with_number() {
    let mut engine = Context::new();

    let init = r#"
        try {
            const bar = Object.create(5);
        } catch (err) {
            err.toString()
        }
        "#;

    let result = forward(&mut engine, init);
    assert_eq!(
        result,
        "\"TypeError: Object prototype may only be an Object or null: 5\""
    );
}

#[test]
#[ignore]
// TODO: to test on __proto__ somehow. __proto__ getter is not working as expected currently
fn object_create_with_function() {
    let mut engine = Context::new();

    let init = r#"
        const x = function (){};
        const bar = Object.create(5);
        bar.__proto__
        "#;

    let result = forward(&mut engine, init);
    assert_eq!(result, "...something on __proto__...");
}

#[test]
fn object_is() {
    let mut engine = Context::new();

    let init = r#"
        var foo = { a: 1};
        var bar = { a: 1};
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "Object.is('foo', 'foo')"), "true");
    assert_eq!(forward(&mut engine, "Object.is('foo', 'bar')"), "false");
    assert_eq!(forward(&mut engine, "Object.is([], [])"), "false");
    assert_eq!(forward(&mut engine, "Object.is(foo, foo)"), "true");
    assert_eq!(forward(&mut engine, "Object.is(foo, bar)"), "false");
    assert_eq!(forward(&mut engine, "Object.is(null, null)"), "true");
    assert_eq!(forward(&mut engine, "Object.is(0, -0)"), "false");
    assert_eq!(forward(&mut engine, "Object.is(-0, -0)"), "true");
    assert_eq!(forward(&mut engine, "Object.is(NaN, 0/0)"), "true");
    assert_eq!(forward(&mut engine, "Object.is()"), "true");
    assert_eq!(forward(&mut engine, "Object.is(undefined)"), "true");
    assert!(engine.global_object().is_global());
    assert!(!engine.global_object().get_field("Object").is_global());
}
#[test]
fn object_has_own_property() {
    let mut engine = Context::new();
    let init = r#"
        let x = { someProp: 1, undefinedProp: undefined, nullProp: null };
    "#;

    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(forward(&mut engine, "x.hasOwnProperty('someProp')"), "true");
    assert_eq!(
        forward(&mut engine, "x.hasOwnProperty('undefinedProp')"),
        "true"
    );
    assert_eq!(forward(&mut engine, "x.hasOwnProperty('nullProp')"), "true");
    assert_eq!(
        forward(&mut engine, "x.hasOwnProperty('hasOwnProperty')"),
        "false"
    );
}

#[test]
fn object_property_is_enumerable() {
    let mut engine = Context::new();
    let init = r#"
        let x = { enumerableProp: 'yes' };
    "#;
    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(
        forward(&mut engine, r#"x.propertyIsEnumerable('enumerableProp')"#),
        "true"
    );
    assert_eq!(
        forward(&mut engine, r#"x.propertyIsEnumerable('hasOwnProperty')"#),
        "false"
    );
    assert_eq!(
        forward(&mut engine, r#"x.propertyIsEnumerable('not_here')"#),
        "false",
    );
    assert_eq!(forward(&mut engine, r#"x.propertyIsEnumerable()"#), "false",)
}

#[test]
fn object_to_string() {
    let mut ctx = Context::new();
    let init = r#"
        let u = undefined;
        let n = null;
        let a = [];
        Array.prototype.toString = Object.prototype.toString;
        let f = () => {};
        Function.prototype.toString = Object.prototype.toString;
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
    eprintln!("{}", forward(&mut ctx, init));
    // TODO: need Function.prototype.call to be implemented
    // assert_eq!(
    //     forward(&mut ctx, "Object.prototype.toString.call(u)"),
    //     "\"[object Undefined]\""
    // );
    // assert_eq!(
    //     forward(&mut ctx, "Object.prototype.toString.call(n)"),
    //     "\"[object Null]\""
    // );
    assert_eq!(forward(&mut ctx, "a.toString()"), "\"[object Array]\"");
    assert_eq!(forward(&mut ctx, "f.toString()"), "\"[object Function]\"");
    assert_eq!(forward(&mut ctx, "b.toString()"), "\"[object Boolean]\"");
    assert_eq!(forward(&mut ctx, "i.toString()"), "\"[object Number]\"");
    assert_eq!(forward(&mut ctx, "s.toString()"), "\"[object String]\"");
    assert_eq!(forward(&mut ctx, "d.toString()"), "\"[object Date]\"");
    assert_eq!(forward(&mut ctx, "re.toString()"), "\"[object RegExp]\"");
    assert_eq!(forward(&mut ctx, "o.toString()"), "\"[object Object]\"");
}

#[test]
fn define_symbol_property() {
    let mut ctx = Context::new();

    let init = r#"
        let obj = {};
        let sym = Symbol("key");
        Object.defineProperty(obj, sym, { value: "val" });
    "#;
    eprintln!("{}", forward(&mut ctx, init));

    assert_eq!(forward(&mut ctx, "obj[sym]"), "\"val\"");
}

#[test]
fn object_define_properties() {
    let mut ctx = Context::new();

    let init = r#"
        const obj = {};

        Object.defineProperties(obj, {
            p: {
                value: 42,
                writable: true
            }
        });
    "#;
    eprintln!("{}", forward(&mut ctx, init));

    assert_eq!(forward(&mut ctx, "obj.p"), "42");
}
