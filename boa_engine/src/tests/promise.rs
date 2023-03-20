use indoc::indoc;

use crate::{job::SimpleJobQueue, Context, run_test_actions_with, TestAction};

#[test]
fn issue_2658() {
    let queue = &SimpleJobQueue::new();
    let context = &mut Context::builder().job_queue(queue).build().unwrap();
    run_test_actions_with([
        TestAction::run(
            indoc! {
                r#"
                    let result1;
                    let result2;
                    async function* agf(a) {
                        for await (m of a) {
                            yield m;
                        }
                    }
                    iterTwo = {
                        [Symbol.asyncIterator]() {
                            return this;
                        },
                        next() {
                            return {
                                value: 5,
                                done: false,
                            };
                        }
                    };
                    const genTwo = agf(iterTwo);
                    genTwo.next().then(v => { console.log(v) });
                    genTwo.next().then(v => { console.log(v) });
                "#
            }
        ),
        TestAction::inspect_context(|ctx| ctx.run_jobs()),
        TestAction::assert("!result1.done"),
        TestAction::assert_eq("result1.value", 5),
        TestAction::assert("!result2.done"),
        TestAction::assert_eq("result2.value", 5),
    ], context)
}