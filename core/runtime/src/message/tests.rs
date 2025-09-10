//! Tests for the `postMessage` extension.

use crate::message;
use crate::message::senders::OnMessageQueueSender;
use crate::test::{TestAction, run_test_actions_with};
use boa_engine::{Context, js_string};
use std::thread;
use std::time::Duration;

/// Create a basic context and allow postMessage from the same context.
#[test]
fn basic() {
    let context = &mut Context::default();

    let sender = OnMessageQueueSender::create(context, 1);
    message::register(sender, None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::harness(),
            TestAction::run(
                r#"
                let latestMessage = null;
                function onMessageQueue(message) {
                    latestMessage = message;
                }

                const message = { "hello": "world" };
                postMessage(message);
                assert(latestMessage === null);
            "#,
            ),
            TestAction::inspect_context(move |context| {
                context.run_jobs().unwrap();
            }),
            TestAction::run(
                r#"
                assert(latestMessage !== null);
                assert(latestMessage !== message);
                assertEq(latestMessage.hello, "world");
            "#,
            ),
        ],
        context,
    );
}

#[test]
fn shared_multi_thread() {
    let (sender, receiver) = std::sync::mpsc::channel::<OnMessageQueueSender>();

    let destination_handle = thread::spawn(move || {
        let context = &mut Context::default();

        // It's important to declare the `onMessageQueue` function before we might
        // receive any messages, as those will be lost.
        run_test_actions_with(
            [
                TestAction::harness(),
                TestAction::run(
                    r#"
                    done = false;
                    function onMessageQueue(message) {
                        assert(message.hello === "world");
                        done = true;
                    }
                "#,
                ),
            ],
            context,
        );

        sender
            .send(OnMessageQueueSender::create(context, 1))
            .unwrap();

        loop {
            thread::sleep(Duration::from_millis(10));
            context.run_jobs().unwrap();

            let global_object = context.global_object();
            if global_object
                .get(js_string!("done"), context)
                .unwrap()
                .as_boolean()
                == Some(true)
            {
                break;
            }
        }
    });

    let source_handle = thread::spawn(move || {
        let context = &mut Context::default();
        let message_sender = receiver.recv().unwrap();
        message::register(message_sender, None, context).unwrap();

        run_test_actions_with(
            [TestAction::run(
                r#"
                    const message = { "hello": "world" };
                    postMessage(message);
                "#,
            )],
            context,
        );
    });

    source_handle.join().unwrap();
    destination_handle.join().unwrap();
}

#[test]
fn shared_array_buffer() {
    let (sender, receiver) = std::sync::mpsc::channel::<OnMessageQueueSender>();

    let destination_handle = thread::spawn(move || {
        let context = &mut Context::default();

        // It's important to declare the `onMessageQueue` function before we might
        // receive any messages, as those will be lost.
        run_test_actions_with(
            [
                TestAction::harness(),
                TestAction::run(
                    r#"
                    done = false;
                    function onMessageQueue(message) {
                        const shared = message.shared;
                        const view = new DataView(shared);
                        while (true) {
                            if (view.getUint32(0) == 1) {
                                view.setUint32(0, 2);
                                done = true;
                                break;
                            }
                        }
                    }
                "#,
                ),
            ],
            context,
        );

        sender
            .send(OnMessageQueueSender::create(context, 1))
            .unwrap();

        loop {
            thread::sleep(Duration::from_millis(10));
            context.run_jobs().unwrap();

            let global_object = context.global_object();
            if global_object
                .get(js_string!("done"), context)
                .unwrap()
                .as_boolean()
                == Some(true)
            {
                break;
            }
        }
    });

    let source_handle = thread::spawn(move || {
        let context = &mut Context::default();

        let message_sender = receiver.recv().unwrap();
        message::register(message_sender, None, context).unwrap();

        run_test_actions_with(
            [TestAction::run(
                r#"
                    const shared = new SharedArrayBuffer(8);
                    const message = { shared };
                    postMessage(message);

                    // Set shared to 1.
                    const view = new DataView(shared);
                    view.setUint32(0, 1);

                    // This would never work if the two contexts are in the same context.
                    while (true) {
                        if (view.getUint32(0) == 2) {
                            view.setUint32(1, 3);
                            break;
                        }
                    }
                "#,
            )],
            context,
        );
    });

    source_handle.join().unwrap();
    destination_handle.join().unwrap();
}
