use crate::{forward, Context};

#[test]
fn construct_empty() {
    let mut context = Context::default();
    let init = r#"
        var empty = new WeakSet();
        "#;
    forward(&mut context, init);
}

#[test]
fn construct_from_array() {
    let mut context = Context::default();
    let init = r#"
        let foo = {};
        let bar = {};
        let weakSet = new WeakSet([foo, bar]);
        "#;
    forward(&mut context, init);
}

#[test]
fn has() {
    let mut context = Context::default();
    let init = r#"
        let foo = {};
        let bar = {};
        let baz = {};
        let weakSet = new WeakSet([foo, bar]);
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "weakSet.has(foo)");
    assert_eq!(result, "true");
    let result = forward(&mut context, "weakSet.has(bar)");
    assert_eq!(result, "true");
    let result = forward(&mut context, "weakSet.has(baz)");
    assert_eq!(result, "false");
}

#[test]
fn add() {
    let mut context = Context::default();
    let init = r#"
        let weakSet = new WeakSet([]);
        let foo = {};       
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "weakSet.add(foo)");
    assert_eq!(result, "{\n\n}");
    let result = forward(&mut context, "weakSet.has(foo)");
    assert_eq!(result, "true");
    let result = forward(&mut context, "weakSet.add(foo)");
    assert_eq!(result, "{\n\n}");
    let result = forward(&mut context, "weakSet.has(foo)");
    assert_eq!(result, "true");
}

#[test]
fn delete() {
    let mut context = Context::default();
    let init = r#"
        let foo = {};
        let bar = {};
        let weakSet = new WeakSet([foo, bar]);
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "weakSet.delete(foo)");
    assert_eq!(result, "true");
    let result = forward(&mut context, "weakSet.has(foo)");
    assert_eq!(result, "false");
    let result = forward(&mut context, "weakSet.has(bar)");
    assert_eq!(result, "true");
    let result = forward(&mut context, "weakSet.delete(foo)");
    assert_eq!(result, "false");
    let result = forward(&mut context, "weakSet.has(foo)");
    assert_eq!(result, "false");
    let result = forward(&mut context, "weakSet.has(bar)");
    assert_eq!(result, "true");
}

#[test]
fn not_a_function() {
    let mut context = Context::default();
    let init = r"
        try {
            let weakSet = WeakSet()
        } catch(e) {
            e.toString()
        }
    ";
    assert_eq!(
        forward(&mut context, init),
        "\"TypeError: calling a builtin WeakSet constructor without new is forbidden\""
    );
}
