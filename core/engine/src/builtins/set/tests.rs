use crate::{run_test_actions, JsNativeErrorKind, TestAction};
use indoc::indoc;

#[test]
fn construct() {
    run_test_actions([
        TestAction::assert_eq("(new Set()).size", 0),
        TestAction::assert_eq("(new Set(['one', 'two'])).size", 2),
    ]);
}

#[test]
fn clone() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                let original = new Set(["one", "two"]);
                let clone = new Set(original);
            "#}),
        TestAction::assert_eq("clone.size", 2),
        TestAction::assert_eq(
            indoc! {r#"
                original.add("three");
                original.size
            "#},
            3,
        ),
        TestAction::assert_eq("clone.size", 2),
    ]);
}

#[test]
fn symbol_iterator() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
                const set1 = new Set();
                set1.add('foo');
                set1.add('bar');
            "#}),
        TestAction::assert(indoc! {r#"
                arrayEquals(
                    Array.from(set1[Symbol.iterator]()),
                    ["foo", "bar"]
                )
            "#}),
    ]);
}

#[test]
fn entries() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
                const set1 = new Set();
                set1.add('foo');
                set1.add('bar')
            "#}),
        TestAction::assert(indoc! {r#"
                arrayEquals(
                    Array.from(set1.entries()),
                    [["foo", "foo"], ["bar", "bar"]]
                )
            "#}),
    ]);
}

#[test]
fn merge() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                let first = new Set(["one", "two"]);
                let second = new Set(["three", "four"]);
                let third = new Set(["four", "five"]);
                let merged1 = new Set([...first, ...second]);
                let merged2 = new Set([...second, ...third]);
            "#}),
        TestAction::assert_eq("merged1.size", 4),
        TestAction::assert_eq("merged2.size", 3),
    ]);
}

#[test]
fn clear() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let set = new Set(["one", "two"]);
            set.clear();
        "#}),
        TestAction::assert_eq("set.size", 0),
    ]);
}

#[test]
fn delete() {
    run_test_actions([
        TestAction::run("let set = new Set(['one', 'two'])"),
        TestAction::assert("set.delete('one')"),
        TestAction::assert_eq("set.size", 1),
        TestAction::assert("!set.delete('one')"),
    ]);
}

#[test]
fn has() {
    run_test_actions([
        TestAction::run("let set = new Set(['one', 'two']);"),
        TestAction::assert("set.has('one')"),
        TestAction::assert("set.has('two')"),
        TestAction::assert("!set.has('three')"),
        TestAction::assert("!set.has()"),
    ]);
}

#[test]
fn values_and_keys() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
                const set1 = new Set();
                set1.add('foo');
                set1.add('bar');
            "#}),
        TestAction::assert(indoc! {r#"
            arrayEquals(
                Array.from(set1.values()),
                ["foo", "bar"]
            )
        "#}),
        TestAction::assert("set1.values == set1.keys"),
    ]);
}

#[test]
fn for_each() {
    run_test_actions([
        TestAction::run(indoc! {r#"
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
            "#}),
        TestAction::assert_eq("value1Sum", 30),
        TestAction::assert_eq("value2Sum", 30),
        TestAction::assert_eq("sizeSum", 9),
    ]);
}

#[test]
fn recursive_display() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                let set = new Set();
                let array = new Array([set]);
                set.add(set);
            "#}),
        TestAction::assert_with_op("set", |v, _| v.display().to_string() == "Set { Set(1) }"),
        TestAction::assert_with_op("set.add(array)", |v, _| {
            v.display().to_string() == "Set { Set(2), Array(1) }"
        }),
    ]);
}

#[test]
fn not_a_function() {
    run_test_actions([TestAction::assert_native_error(
        "Set()",
        JsNativeErrorKind::Type,
        "calling a builtin Set constructor without new is forbidden",
    )]);
}
