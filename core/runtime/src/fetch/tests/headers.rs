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
