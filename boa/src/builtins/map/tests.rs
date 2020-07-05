use crate::{exec::Interpreter, forward, realm::Realm};

#[test]
fn construct_empty() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var empty = new Map();
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "empty.size");
    assert_eq!(result, "0");
}

#[test]
fn construct_from_array() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        let map = new Map([["1", "one"], ["2", "two"]]);
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map.size");
    assert_eq!(result, "2");
}

#[test]
fn clone() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        let original = new Map([["1", "one"], ["2", "two"]]);
        let clone = new Map(original);
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "clone.size");
    assert_eq!(result, "2");
    let result = forward(
        &mut engine,
        r#"
        original.set("3", "three");
        original.size"#,
    );
    assert_eq!(result, "3");
    let result = forward(&mut engine, "clone.size");
    assert_eq!(result, "2");
}

#[test]
fn merge() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        let first = new Map([["1", "one"], ["2", "two"]]);
        let second = new Map([["2", "second two"], ["3", "three"]]);
        let third = new Map([["4", "four"], ["5", "five"]]);
        let merged1 = new Map([...first, ...second]);
        let merged2 = new Map([...second, ...third]);
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "merged1.size");
    assert_eq!(result, "3");
    let result = forward(&mut engine, "merged1.get('2')");
    assert_eq!(result, "second two");
    let result = forward(&mut engine, "merged2.size");
    assert_eq!(result, "4");
}

#[test]
fn get() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        let map = new Map([["1", "one"], ["2", "two"]]);
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map.get('1')");
    assert_eq!(result, "one");
    let result = forward(&mut engine, "map.get('2')");
    assert_eq!(result, "two");
    let result = forward(&mut engine, "map.get('3')");
    assert_eq!(result, "undefined");
    let result = forward(&mut engine, "map.get()");
    assert_eq!(result, "undefined");
}

#[test]
fn set() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        let map = new Map();
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map.set();map.size");
    assert_eq!(result, "1");
    let result = forward(&mut engine, "map.set('1', 'one');map.size");
    assert_eq!(result, "2");
    let result = forward(&mut engine, "map.set('2');map.size");
    assert_eq!(result, "3");
}

#[test]
fn clear() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        let map = new Map([["1", "one"], ["2", "two"]]);
        map.clear();
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map.size");
    assert_eq!(result, "0");
}

#[test]
fn delete() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        let map = new Map([["1", "one"], ["2", "two"]]);
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map.delete('1')");
    assert_eq!(result, "true");
    let result = forward(&mut engine, "map.size");
    assert_eq!(result, "1");
    let result = forward(&mut engine, "map.delete('1')");
    assert_eq!(result, "false");
}

#[test]
fn has() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        let map = new Map([["1", "one"]]);
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map.has()");
    assert_eq!(result, "false");
    let result = forward(&mut engine, "map.has('1')");
    assert_eq!(result, "true");
    let result = forward(&mut engine, "map.has('2')");
    assert_eq!(result, "false");
}

#[test]
fn for_each() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        let map = new Map([[1, 5], [2, 10], [3, 15]]);
        let valueSum = 0;
        let keySum = 0;
        let sizeSum = 0;
        function callingCallback(value, key, map) {
            valueSum += value;
            keySum += key;
            sizeSum += map.size;
        }
        map.forEach(callingCallback);
        "#;
    forward(&mut engine, init);
    assert_eq!(forward(&mut engine, "valueSum"), "30");
    assert_eq!(forward(&mut engine, "keySum"), "6");
    assert_eq!(forward(&mut engine, "sizeSum"), "9");
}
