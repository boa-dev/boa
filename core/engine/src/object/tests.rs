use crate::{TestAction, run_test_actions};
use indoc::indoc;

#[test]
fn array_prototype_map_edge_cases() {
    run_test_actions([
        TestAction::run_harness(),

        // Empty array
        TestAction::assert(r#"arrayEquals([].map(x => x), [])"#),

        // Callback returning undefined
        TestAction::assert(
            r#"arrayEquals([1, 2, 3].map(() => undefined), [undefined, undefined, undefined])"#
        ),

        // Sparse array (check length + defined values)
        TestAction::run(indoc! {r#"
            let arr = [1, , 3];
            let result = arr.map(x => x);
        "#}),
        TestAction::assert("result.length === 3"),
        TestAction::assert("result[0] === 1"),
        TestAction::assert("!(1 in result)"),
        TestAction::assert("result[2] === 3"),

        // Identity mapping
        TestAction::assert(r#"arrayEquals([1, 2, 3].map(x => x), [1, 2, 3])"#),
    ]);
}