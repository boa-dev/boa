use crate::test::{TestAction, run_test_actions_with};
use crate::{ConsoleState, Logger, RuntimeExtension};
use boa_engine::{Context, JsError, JsResult};
use boa_gc::{Gc, GcRefCell};
use indoc::indoc;

/// A logger that records all log messages, used to observe job ordering.
#[derive(Clone, Debug, Default, boa_engine::Trace, boa_engine::Finalize)]
struct RecordingLogger {
    log: Gc<GcRefCell<String>>,
}

impl Logger for RecordingLogger {
    fn log(&self, msg: String, state: &ConsoleState, _: &mut Context) -> JsResult<()> {
        use std::fmt::Write;
        let indent = state.indent();
        writeln!(self.log.borrow_mut(), "{msg:>indent$}").map_err(JsError::from_rust)
    }

    fn info(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    fn warn(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    fn error(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }
}

#[test]
fn queue_microtask() {
    let context = &mut Context::default();
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
