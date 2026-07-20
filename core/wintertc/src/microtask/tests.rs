use crate::test::{TestAction, run_test_actions_with};
use boa_engine::{Context, js_str};
use indoc::indoc;

#[test]
fn queue_microtask() {
    let context = &mut Context::default();
    crate::microtask::register(None, context).unwrap();

    run_test_actions_with(
        [
            // Record execution order in a global array instead of relying on
            // `console`, which is not part of this crate's test surface.
            TestAction::run(indoc! {r#"
                order = [];
                order.push(1);
                queueMicrotask(() => order.push(2));
                order.push(3);
                queueMicrotask(() => {
                    order.push(4);
                    queueMicrotask(() => {
                        order.push(5);
                        queueMicrotask(() => order.push(6));
                        order.push(7);
                    });
                    order.push(8);
                });
                order.push(9);
            "#}),
            TestAction::inspect_context(|ctx| {
                ctx.run_jobs().unwrap();
            }),
            TestAction::inspect_context(|ctx| {
                let order = ctx.global_object().get(js_str!("order"), ctx).unwrap();
                let order = order.as_object().unwrap();

                assert_eq!(
                    order.get(js_str!("length"), ctx).unwrap().as_i32(),
                    Some(9),
                    "all nine pushes must run",
                );

                // Synchronous pushes run first (1, 3, 9); the queued microtasks
                // then drain in FIFO order, with nested microtasks appended as
                // they are enqueued.
                for (i, expected) in [1, 3, 9, 2, 4, 8, 5, 7, 6].into_iter().enumerate() {
                    assert_eq!(
                        order.get(i, ctx).unwrap().as_i32(),
                        Some(expected),
                        "order[{i}] mismatch",
                    );
                }
            }),
        ],
        context,
    );
}
