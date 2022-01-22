use crate::{forward, Context};

#[test]
fn construct_empty() {
    let mut context = Context::default();
    let init = r#"
        var empty = new Set();
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "empty.size");
    assert_eq!(result, "0");
}

#[test]
fn construct_from_array() {
    let mut context = Context::default();
    let init = r#"
        let set = new Set(["one", "two"]);
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "set.size");
    assert_eq!(result, "2");
}

#[test]
fn clone() {
    let mut context = Context::default();
    let init = r#"
        let original = new Set(["one", "two"]);
        let clone = new Set(original);
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "clone.size");
    assert_eq!(result, "2");
    let result = forward(
        &mut context,
        r#"
        original.add("three");
        original.size"#,
    );
    assert_eq!(result, "3");
    let result = forward(&mut context, "clone.size");
    assert_eq!(result, "2");
}

#[test]
fn symbol_iterator() {
    let mut context = Context::default();
    let init = r#"
        const set1 = new Set();
        set1.add('foo');
        set1.add('bar');
        const iterator = set1[Symbol.iterator]();
        let item1 = iterator.next();
        let item2 = iterator.next();
        let item3 = iterator.next();
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "item1.value");
    assert_eq!(result, "\"foo\"");
    let result = forward(&mut context, "item1.done");
    assert_eq!(result, "false");
    let result = forward(&mut context, "item2.value");
    assert_eq!(result, "\"bar\"");
    let result = forward(&mut context, "item2.done");
    assert_eq!(result, "false");
    let result = forward(&mut context, "item3.value");
    assert_eq!(result, "undefined");
    let result = forward(&mut context, "item3.done");
    assert_eq!(result, "true");
}

#[test]
fn entries() {
    let mut context = Context::default();
    let init = r#"
        const set1 = new Set();
        set1.add('foo');
        set1.add('bar');
        const entriesIterator = set1.entries();
        let item1 = entriesIterator.next();
        let item2 = entriesIterator.next();
        let item3 = entriesIterator.next();
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "item1.value.length");
    assert_eq!(result, "2");
    let result = forward(&mut context, "item1.value[0]");
    assert_eq!(result, "\"foo\"");
    let result = forward(&mut context, "item1.value[1]");
    assert_eq!(result, "\"foo\"");
    let result = forward(&mut context, "item1.done");
    assert_eq!(result, "false");
    let result = forward(&mut context, "item2.value.length");
    assert_eq!(result, "2");
    let result = forward(&mut context, "item2.value[0]");
    assert_eq!(result, "\"bar\"");
    let result = forward(&mut context, "item2.value[1]");
    assert_eq!(result, "\"bar\"");
    let result = forward(&mut context, "item2.done");
    assert_eq!(result, "false");
    let result = forward(&mut context, "item3.value");
    assert_eq!(result, "undefined");
    let result = forward(&mut context, "item3.done");
    assert_eq!(result, "true");
}

#[test]
fn merge() {
    let mut context = Context::default();
    let init = r#"
        let first = new Set(["one", "two"]);
        let second = new Set(["three", "four"]);
        let third = new Set(["four", "five"]);
        let merged1 = new Set([...first, ...second]);
        let merged2 = new Set([...second, ...third]);
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "merged1.size");
    assert_eq!(result, "4");
    let result = forward(&mut context, "merged2.size");
    assert_eq!(result, "3");
}

#[test]
fn clear() {
    let mut context = Context::default();
    let init = r#"
        let set = new Set(["one", "two"]);
        set.clear();
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "set.size");
    assert_eq!(result, "0");
}

#[test]
fn delete() {
    let mut context = Context::default();
    let init = r#"
        let set = new Set(["one", "two"]);
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "set.delete('one')");
    assert_eq!(result, "true");
    let result = forward(&mut context, "set.size");
    assert_eq!(result, "1");
    let result = forward(&mut context, "set.delete('one')");
    assert_eq!(result, "false");
}

#[test]
fn has() {
    let mut context = Context::default();
    let init = r#"
        let set = new Set(["one", "two"]);
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "set.has('one')");
    assert_eq!(result, "true");
    let result = forward(&mut context, "set.has('two')");
    assert_eq!(result, "true");
    let result = forward(&mut context, "set.has('three')");
    assert_eq!(result, "false");
    let result = forward(&mut context, "set.has()");
    assert_eq!(result, "false");
}

#[test]
fn values_and_keys() {
    let mut context = Context::default();
    let init = r#"
        const set1 = new Set();
        set1.add('foo');
        set1.add('bar');
        const valuesIterator = set1.values();
        let item1 = valuesIterator.next();
        let item2 = valuesIterator.next();
        let item3 = valuesIterator.next();
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "item1.value");
    assert_eq!(result, "\"foo\"");
    let result = forward(&mut context, "item1.done");
    assert_eq!(result, "false");
    let result = forward(&mut context, "item2.value");
    assert_eq!(result, "\"bar\"");
    let result = forward(&mut context, "item2.done");
    assert_eq!(result, "false");
    let result = forward(&mut context, "item3.value");
    assert_eq!(result, "undefined");
    let result = forward(&mut context, "item3.done");
    assert_eq!(result, "true");
    let result = forward(&mut context, "set1.values == set1.keys");
    assert_eq!(result, "true");
}

#[test]
fn for_each() {
    let mut context = Context::default();
    let init = r#"
        let set = new Set([5, 10, 15]);
        let value1Sum = 0;
        let value2Sum = 0;
        let sizeSum = 0;
        function callingCallback(value1, value2, set) {
            value1Sum += value1;
            value2Sum += value2;
            sizeSum += set.size;
        }
        set.forEach(callingCallback);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "value1Sum"), "30");
    assert_eq!(forward(&mut context, "value2Sum"), "30");
    assert_eq!(forward(&mut context, "sizeSum"), "9");
}

#[test]
fn recursive_display() {
    let mut context = Context::default();
    let init = r#"
        let set = new Set();
        let array = new Array([set]);
        set.add(set);
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "set");
    assert_eq!(result, "Set { Set(1) }");
    let result = forward(&mut context, "set.add(array)");
    assert_eq!(result, "Set { Set(2), Array(1) }");
}

#[test]
fn not_a_function() {
    let mut context = Context::default();
    let init = r"
        try {
            let set = Set()
        } catch(e) {
            e.toString()
        }
    ";
    assert_eq!(
        forward(&mut context, init),
        "\"TypeError: calling a builtin Set constructor without new is forbidden\""
    );
}
