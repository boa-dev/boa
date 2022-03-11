use crate::{forward, Context};

#[test]
fn apply() {
    let mut context = Context::default();

    let init = r#"
        var called = {};
        function f(n) { called.result = n };
        Reflect.apply(f, undefined, [42]);
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "called.result"), "42");
}

#[test]
fn construct() {
    let mut context = Context::default();

    let init = r#"
        var called = {};
        function f(n) { called.result = n };
        Reflect.construct(f, [42]);
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "called.result"), "42");
}

#[test]
fn define_property() {
    let mut context = Context::default();

    let init = r#"
        let obj = {};
        Reflect.defineProperty(obj, 'p', { value: 42 });
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "obj.p"), "42");
}

#[test]
fn delete_property() {
    let mut context = Context::default();

    let init = r#"
        let obj = { p: 42 };
        let deleted = Reflect.deleteProperty(obj, 'p');
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "obj.p"), "undefined");
    assert_eq!(forward(&mut context, "deleted"), "true");
}

#[test]
fn get() {
    let mut context = Context::default();

    let init = r#"
        let obj = { p: 42 }
        let p = Reflect.get(obj, 'p');
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "p"), "42");
}

#[test]
fn get_own_property_descriptor() {
    let mut context = Context::default();

    let init = r#"
        let obj = { p: 42 };
        let desc = Reflect.getOwnPropertyDescriptor(obj, 'p');
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "desc.value"), "42");
}

#[test]
fn get_prototype_of() {
    let mut context = Context::default();

    let init = r#"
        function F() { this.p = 42 };
        let f = new F();
        let proto = Reflect.getPrototypeOf(f);
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "proto.constructor.name"), "\"F\"");
}

#[test]
fn has() {
    let mut context = Context::default();

    let init = r#"
        let obj = { p: 42 };
        let hasP = Reflect.has(obj, 'p');
        let hasP2 = Reflect.has(obj, 'p2');
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "hasP"), "true");
    assert_eq!(forward(&mut context, "hasP2"), "false");
}

#[test]
fn is_extensible() {
    let mut context = Context::default();

    let init = r#"
        let obj = { p: 42 };
        let isExtensible = Reflect.isExtensible(obj);
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "isExtensible"), "true");
}

#[test]
fn own_keys() {
    let mut context = Context::default();

    let init = r#"
        let obj = { p: 42 };
        let ownKeys = Reflect.ownKeys(obj);
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "ownKeys"), r#"[ "p" ]"#);
}

#[test]
fn prevent_extensions() {
    let mut context = Context::default();

    let init = r#"
        let obj = { p: 42 };
        let r = Reflect.preventExtensions(obj);
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "r"), "true");
}

#[test]
fn set() {
    let mut context = Context::default();

    let init = r#"
        let obj = {};
        Reflect.set(obj, 'p', 42);
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "obj.p"), "42");
}

#[test]
fn set_prototype_of() {
    let mut context = Context::default();

    let init = r#"
        function F() { this.p = 42 };
        let obj = {}
        Reflect.setPrototypeOf(obj, F);
        let p = Reflect.getPrototypeOf(obj);
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "p.name"), "\"F\"");
}
