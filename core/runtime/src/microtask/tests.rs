use crate::RuntimeExtension;
use crate::console::tests::RecordingLogger;
use crate::test::{TestAction, run_test_actions_with};
use boa_engine::Context;
use indoc::indoc;

#[test]
fn queue_microtask() {
    let context = &Context::default();
    crate::microtask::register(None, context).unwrap();
    let logger = RecordingLogger::default();
    crate::extensions::ConsoleExtension(logger.clone())
        .register(None, context)
        .unwrap();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                console.log(1);
                queueMicrotask(() => console.log(2));
                console.log(3);
                queueMicrotask(() => {
                    console.log(4);
                    queueMicrotask(() => {
                        console.log(5);
                        queueMicrotask(() => console.log(6));
                        console.log(7);
                    });
                    console.log(8);
                });
                console.log(9);
            "#}),
            TestAction::inspect_context(|context| {
                context.run_jobs().unwrap();
            }),
        ],
        context,
    );

    let logs = logger.log.borrow().clone();
    assert_eq!(
        logs,
        indoc! { r#"
            1
            3
            9
            2
            4
            8
            5
            7
            6
        "# }
    );
}
