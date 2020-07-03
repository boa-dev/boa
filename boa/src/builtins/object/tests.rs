use crate::{exec::Interpreter, forward, realm::Realm};

#[test]
fn object_create() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let init = r#"
        const foo = { a: 5 };
        const bar = Object.create(foo);
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "bar.a"), "5");
    assert_eq!(forward(&mut engine, "Object.create.length"), "1");
}

#[test]
fn object_is() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

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
}
#[test]
fn object_has_own_property() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
