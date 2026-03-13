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

#[test]
fn headers_iterators() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const headers = new Headers([["a", "1"], ["b", "2"]]);
                
                // entries()
                const entriesList = [...headers.entries()];
                assertEq(entriesList.length, 2);
                assertEq(entriesList[0][0], "a");
                assertEq(entriesList[0][1], "1");
                
                // keys()
                const keysList = [...headers.keys()];
                assertEq(keysList.length, 2);
                assertEq(keysList[0], "a");
                assertEq(keysList[1], "b");
                
                // values()
                const valuesList = [...headers.values()];
                assertEq(valuesList.length, 2);
                assertEq(valuesList[0], "1");
                assertEq(valuesList[1], "2");
                
                // .next() directly
                const it = headers.entries();
                const first = it.next();
                assertEq(first.done, false);
                assertEq(first.value[0], "a");
            "#,
        ),
    ]);
}
