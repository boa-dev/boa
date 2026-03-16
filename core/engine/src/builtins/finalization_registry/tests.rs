use indoc::indoc;

use crate::{TestAction, run_test_actions};

#[test]
fn finalization_registry_simple() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let counter = 0;
            const registry = new FinalizationRegistry(() => {
                counter++;
            });

            registry.register(["foo"]);
        "#}),
        TestAction::assert_eq("counter", 0),
        TestAction::inspect_context(|_| boa_gc::force_collect()),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("counter", 1),
    ]);
}
