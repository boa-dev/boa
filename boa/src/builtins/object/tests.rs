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
// to test on __proto__ somehow. __proto__ getter is not working as expected currently
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
