use crate::{forward, Context};

#[test]
fn construct_empty() {
    let mut engine = Context::new();
    let init = r#"
        var empty = new Map();
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "empty.size");
    assert_eq!(result, "0");
}

#[test]
fn construct_from_array() {
    let mut engine = Context::new();
    let init = r#"
        let map = new Map([["1", "one"], ["2", "two"]]);
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map.size");
    assert_eq!(result, "2");
}

#[test]
fn clone() {
    let mut engine = Context::new();
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
fn symbol_iterator() {
    let mut engine = Context::new();
    let init = r#"
        const map1 = new Map();
        map1.set('0', 'foo');
        map1.set(1, 'bar');
        const iterator = map1[Symbol.iterator]();
        let item1 = iterator.next();
        let item2 = iterator.next();
        let item3 = iterator.next();
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "item1.value.length");
    assert_eq!(result, "2");
    let result = forward(&mut engine, "item1.value[0]");
    assert_eq!(result, "\"0\"");
    let result = forward(&mut engine, "item1.value[1]");
    assert_eq!(result, "\"foo\"");
    let result = forward(&mut engine, "item1.done");
    assert_eq!(result, "false");
    let result = forward(&mut engine, "item2.value.length");
    assert_eq!(result, "2");
    let result = forward(&mut engine, "item2.value[0]");
    assert_eq!(result, "1");
    let result = forward(&mut engine, "item2.value[1]");
    assert_eq!(result, "\"bar\"");
    let result = forward(&mut engine, "item2.done");
    assert_eq!(result, "false");
    let result = forward(&mut engine, "item3.value");
    assert_eq!(result, "undefined");
    let result = forward(&mut engine, "item3.done");
    assert_eq!(result, "true");
}

// Should behave the same as symbol_iterator
#[test]
fn entries() {
    let mut engine = Context::new();
    let init = r#"
        const map1 = new Map();
        map1.set('0', 'foo');
        map1.set(1, 'bar');
        const entriesIterator = map1.entries();
        let item1 = entriesIterator.next();
        let item2 = entriesIterator.next();
        let item3 = entriesIterator.next();
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "item1.value.length");
    assert_eq!(result, "2");
    let result = forward(&mut engine, "item1.value[0]");
    assert_eq!(result, "\"0\"");
    let result = forward(&mut engine, "item1.value[1]");
    assert_eq!(result, "\"foo\"");
    let result = forward(&mut engine, "item1.done");
    assert_eq!(result, "false");
    let result = forward(&mut engine, "item2.value.length");
    assert_eq!(result, "2");
    let result = forward(&mut engine, "item2.value[0]");
    assert_eq!(result, "1");
    let result = forward(&mut engine, "item2.value[1]");
    assert_eq!(result, "\"bar\"");
    let result = forward(&mut engine, "item2.done");
    assert_eq!(result, "false");
    let result = forward(&mut engine, "item3.value");
    assert_eq!(result, "undefined");
    let result = forward(&mut engine, "item3.done");
    assert_eq!(result, "true");
}

#[test]
fn merge() {
    let mut engine = Context::new();
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
    assert_eq!(result, "\"second two\"");
    let result = forward(&mut engine, "merged2.size");
    assert_eq!(result, "4");
}

#[test]
fn get() {
    let mut engine = Context::new();
    let init = r#"
        let map = new Map([["1", "one"], ["2", "two"]]);
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map.get('1')");
    assert_eq!(result, "\"one\"");
    let result = forward(&mut engine, "map.get('2')");
    assert_eq!(result, "\"two\"");
    let result = forward(&mut engine, "map.get('3')");
    assert_eq!(result, "undefined");
    let result = forward(&mut engine, "map.get()");
    assert_eq!(result, "undefined");
}

#[test]
fn set() {
    let mut engine = Context::new();
    let init = r#"
        let map = new Map();
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map.set()");
    assert_eq!(result, "Map { undefined → undefined }");
    let result = forward(&mut engine, "map.set('1', 'one')");
    assert_eq!(result, "Map { undefined → undefined, \"1\" → \"one\" }");
    let result = forward(&mut engine, "map.set('2')");
    assert_eq!(
        result,
        "Map { undefined → undefined, \"1\" → \"one\", \"2\" → undefined }"
    );
}

#[test]
fn clear() {
    let mut engine = Context::new();
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
    let mut engine = Context::new();
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
    let mut engine = Context::new();
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
fn keys() {
    let mut engine = Context::new();
    let init = r#"
        const map1 = new Map();
        map1.set('0', 'foo');
        map1.set(1, 'bar');
        const keysIterator = map1.keys();
        let item1 = keysIterator.next();
        let item2 = keysIterator.next();
        let item3 = keysIterator.next();
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "item1.value");
    assert_eq!(result, "\"0\"");
    let result = forward(&mut engine, "item1.done");
    assert_eq!(result, "false");
    let result = forward(&mut engine, "item2.value");
    assert_eq!(result, "1");
    let result = forward(&mut engine, "item2.done");
    assert_eq!(result, "false");
    let result = forward(&mut engine, "item3.value");
    assert_eq!(result, "undefined");
    let result = forward(&mut engine, "item3.done");
    assert_eq!(result, "true");
}

#[test]
fn for_each() {
    let mut engine = Context::new();
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

#[test]
fn values() {
    let mut engine = Context::new();
    let init = r#"
        const map1 = new Map();
        map1.set('0', 'foo');
        map1.set(1, 'bar');
        const valuesIterator = map1.values();
        let item1 = valuesIterator.next();
        let item2 = valuesIterator.next();
        let item3 = valuesIterator.next();
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "item1.value");
    assert_eq!(result, "\"foo\"");
    let result = forward(&mut engine, "item1.done");
    assert_eq!(result, "false");
    let result = forward(&mut engine, "item2.value");
    assert_eq!(result, "\"bar\"");
    let result = forward(&mut engine, "item2.done");
    assert_eq!(result, "false");
    let result = forward(&mut engine, "item3.value");
    assert_eq!(result, "undefined");
    let result = forward(&mut engine, "item3.done");
    assert_eq!(result, "true");
}

#[test]
fn modify_key() {
    let mut engine = Context::new();
    let init = r#"
        let obj = new Object();
        let map = new Map([[obj, "one"]]);
        obj.field = "Value";
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map.get(obj)");
    assert_eq!(result, "\"one\"");
}

#[test]
fn order() {
    let mut engine = Context::new();
    let init = r#"
        let map = new Map([[1, "one"]]);
        map.set(2, "two");
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map");
    assert_eq!(result, "Map { 1 → \"one\", 2 → \"two\" }");
    let result = forward(&mut engine, "map.set(1, \"five\");map");
    assert_eq!(result, "Map { 1 → \"five\", 2 → \"two\" }");
    let result = forward(&mut engine, "map.set();map");
    assert_eq!(
        result,
        "Map { 1 → \"five\", 2 → \"two\", undefined → undefined }"
    );
    let result = forward(&mut engine, "map.delete(2);map");
    assert_eq!(result, "Map { 1 → \"five\", undefined → undefined }");
    let result = forward(&mut engine, "map.set(2, \"two\");map");
    assert_eq!(
        result,
        "Map { 1 → \"five\", undefined → undefined, 2 → \"two\" }"
    );
}

#[test]
fn recursive_display() {
    let mut engine = Context::new();
    let init = r#"
        let map = new Map();
        let array = new Array([map]);
        map.set("y", map);
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "map");
    assert_eq!(result, "Map { \"y\" → Map(1) }");
    let result = forward(&mut engine, "map.set(\"z\", array)");
    assert_eq!(result, "Map { \"y\" → Map(2), \"z\" → Array(1) }");
}

#[test]
fn not_a_function() {
    let mut engine = Context::new();
    let init = r"
        try {
            let map = Map()
        } catch(e) {
            e.toString()
        }
    ";
    assert_eq!(
        forward(&mut engine, init),
        "\"TypeError: function object is not callable\""
    );
}
