use super::TestFetcher;
use crate::test::{TestAction, run_test_actions};

fn register(ctx: &mut boa_engine::Context) {
    crate::fetch::register(TestFetcher::default(), None, ctx).expect("failed to register fetch");
}

#[test]
fn headers_are_iterable() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const headers = new Headers([["x", "y"]]);
                const entries = [...headers];
                assertEq(entries.length, 1);
                assertEq(entries[0][0], "x");
                assertEq(entries[0][1], "y");

                const map = new Map(headers);
                assertEq(map.get("x"), "y");
            "#,
        ),
    ]);
}

// Regression tests for https://github.com/boa-dev/boa/issues/4989
// Headers.entries(), keys(), and values() should return proper iterators.

#[test]
fn headers_entries_returns_iterator() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const h = new Headers([["a", "b"], ["c", "d"]]);
                const it = h.entries();
                assertEq(Array.isArray(it), false);
                assertEq(typeof it.next, "function");
                const first = it.next();
                assertEq(first.done, false);
                assertEq(first.value[0], "a");
                assertEq(first.value[1], "b");
                const second = it.next();
                assertEq(second.done, false);
                assertEq(second.value[0], "c");
                assertEq(second.value[1], "d");
                const end = it.next();
                assertEq(end.done, true);
            "#,
        ),
    ]);
}

#[test]
fn headers_keys_returns_iterator() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const h = new Headers([["a", "b"], ["c", "d"]]);
                const it = h.keys();
                assertEq(typeof it.next, "function");
                const first = it.next();
                assertEq(first.done, false);
                assertEq(first.value, "a");
                const second = it.next();
                assertEq(second.done, false);
                assertEq(second.value, "c");
                const end = it.next();
                assertEq(end.done, true);
            "#,
        ),
    ]);
}

#[test]
fn headers_values_returns_iterator() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const h = new Headers([["a", "b"], ["c", "d"]]);
                const it = h.values();
                assertEq(typeof it.next, "function");
                const first = it.next();
                assertEq(first.done, false);
                assertEq(first.value, "b");
                const second = it.next();
                assertEq(second.done, false);
                assertEq(second.value, "d");
                const end = it.next();
                assertEq(end.done, true);
            "#,
        ),
    ]);
}

#[test]
fn headers_symbol_iterator_still_works() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const h = new Headers([["a", "b"], ["c", "d"]]);
                const it = h[Symbol.iterator]();
                assertEq(typeof it.next, "function");
                const first = it.next();
                assertEq(first.done, false);
                assertEq(first.value[0], "a");
                assertEq(first.value[1], "b");
                const collected = [...h];
                assertEq(collected.length, 2);
            "#,
        ),
    ]);
}

#[test]
fn headers_for_of_with_entries() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const h = new Headers([["a", "b"], ["c", "d"]]);
                const result = [];
                for (const [k, v] of h.entries()) {
                    result.push(k + "=" + v);
                }
                assertEq(result.length, 2);
                assertEq(result[0], "a=b");
                assertEq(result[1], "c=d");
            "#,
        ),
    ]);
}
