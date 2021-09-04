use crate::exec;

#[test]
fn spread_shallow_clone() {
    let scenario = r#"
        var a = { x: {} };
        var aClone = { ...a };

        a.x === aClone.x
    "#;
    assert_eq!(&exec(scenario), "true");
}

#[test]
fn spread_merge() {
    let scenario = r#"
        var a = { x: 1, y: 2 };
        var b = { x: -1, z: -3, ...a };

        (b.x === 1) && (b.y === 2) && (b.z === -3)
    "#;
    assert_eq!(&exec(scenario), "true");
}

#[test]
fn spread_overriding_properties() {
    let scenario = r#"
        var a = { x: 0, y: 0 };
        var aWithOverrides = { ...a, ...{ x: 1, y: 2 } };

        (aWithOverrides.x === 1) && (aWithOverrides.y === 2)
    "#;
    assert_eq!(&exec(scenario), "true");
}

#[test]
fn spread_getters_in_initializer() {
    let scenario = r#"
        var a = { x: 42 };
        var aWithXGetter = { ...a, get x() { throw new Error('not thrown yet') } };
    "#;
    assert_eq!(&exec(scenario), "undefined");
}

#[test]
fn spread_getters_in_object() {
    let scenario = r#"
        var a = { x: 42 };
        var aWithXGetter = { ...a, ... { get x() { throw new Error('not thrown yet') } } };
    "#;
    assert_eq!(&exec(scenario), "\"Error\": \"not thrown yet\"");
}

#[test]
fn spread_setters() {
    let scenario = r#"
        var z = { set x(nexX) { throw new Error() }, ... { x: 1 } };
    "#;
    assert_eq!(&exec(scenario), "undefined");
}

#[test]
fn spread_null_and_undefined_ignored() {
    let scenario = r#"
        var a = { ...null, ...undefined };
        var count = 0;

        for (key in a) { count++; }

        count === 0
    "#;

    assert_eq!(&exec(scenario), "true");
}

#[test]
fn fmt() {
    super::super::test_formatting(
        r#"
        let other = {
            c: 10,
        };
        let inst = {
            val: 5,
            b: "hello world",
            nested: {
                a: 5,
                b: 6,
            },
            ...other,
            say_hi: function() {
                console.log("hello!");
            },
            get a() {
                return this.val + 1;
            },
            set a(new_value) {
                this.val = new_value;
            },
            say_hello(msg) {
                console.log("hello " + msg);
            },
        };
        inst.a = 20;
        inst.a;
        inst.say_hello("humans");
        "#,
    );
}
