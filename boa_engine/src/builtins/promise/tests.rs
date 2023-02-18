use crate::{job::SimpleJobQueue, run_test_actions_with, Context, Runtime, TestAction};
use indoc::indoc;

#[test]
fn promise() {
    let rt = &Runtime::default();
    let queue = SimpleJobQueue::new();
    let context = &mut Context::builder(rt).job_queue(&queue).build().unwrap();
    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                    let count = 0;
                    const promise = new Promise((resolve, reject) => {
                        count += 1;
                        resolve(undefined);
                    }).then((_) => (count += 1));
                    count += 1;
                "#}),
            TestAction::assert_eq("count", 2),
            #[allow(clippy::redundant_closure_for_method_calls)]
            TestAction::inspect_context(|ctx| ctx.run_jobs()),
            TestAction::assert_eq("count", 3),
        ],
        context,
    );
}
