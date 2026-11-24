use crate::test::{TestAction, run_test_actions_with};
use boa_engine::Context;
use indoc::indoc;

#[test]
fn structured_clone_data_view() {
    let context = &mut Context::default();
    crate::clone::register(None, context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            const buffer = new ArrayBuffer(16);
            const view = new DataView(buffer, 4, 8);
            view.setInt32(0, 42);

            const cloned = structuredClone(view);
            if (!(cloned instanceof DataView)) {
                throw new Error("cloned is not an instance of DataView");
            }
            if (cloned === view) {
                throw new Error("cloned is identical to view");
            }
            if (cloned.buffer === view.buffer) {
                throw new Error("cloned buffer is identical to view buffer");
            }
            if (cloned.byteOffset !== 4) {
                throw new Error("cloned.byteOffset !== 4: " + cloned.byteOffset);
            }
            if (cloned.byteLength !== 8) {
                throw new Error("cloned.byteLength !== 8: " + cloned.byteLength);
            }
            if (cloned.getInt32(0) !== 42) {
                throw new Error("cloned.getInt32(0) !== 42: " + cloned.getInt32(0));
            }

            // Mutation on original should not affect the clone
            view.setInt32(0, 99);
            if (view.getInt32(0) !== 99) {
                throw new Error("view.getInt32(0) !== 99");
            }
            if (cloned.getInt32(0) !== 42) {
                throw new Error("cloned was mutated when view was mutated");
            }
        "#})],
        context,
    );
}

#[test]
fn structured_clone_data_view_shared_buffer() {
    let context = &mut Context::default();
    crate::clone::register(None, context).unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            const buffer = new ArrayBuffer(8);
            const view1 = new DataView(buffer, 0, 4);
            const view2 = new DataView(buffer, 4, 4);
            const pair = structuredClone([view1, view2]);

            if (!(pair[0] instanceof DataView) || !(pair[1] instanceof DataView)) {
                throw new Error("cloned pair elements are not DataView instances");
            }
            if (pair[0].buffer !== pair[1].buffer) {
                throw new Error("cloned views do not share the underlying ArrayBuffer");
            }
            if (pair[0].byteOffset !== 0 || pair[1].byteOffset !== 4) {
                throw new Error("cloned views byteOffset mismatch");
            }
        "#})],
        context,
    );
}
