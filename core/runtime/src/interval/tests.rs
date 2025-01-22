use crate::interval;
use crate::test::{run_test_actions_with, TestAction};
use boa_engine::context::{ContextBuilder, HostHooks};
use boa_engine::{js_str, Context};
use std::cell::RefCell;
use std::rc::Rc;

/// A simple clock that can be used for testing.
#[derive(Clone)]
struct TestClockHooks {
    time: Rc<RefCell<i64>>,
}

impl Default for TestClockHooks {
    fn default() -> Self {
        Self {
            time: Rc::new(RefCell::new(1_000_000)),
        }
    }
}

impl TestClockHooks {
    /// Move the clock forwards a number of milliseconds.
    fn forward(&self, ms: i64) {
        *self.time.borrow_mut() += ms;
    }
}

impl HostHooks for TestClockHooks {
    fn utc_now(&self) -> i64 {
        *self.time.borrow()
    }
}

fn create_context(hooks: Rc<TestClockHooks>) -> Context {
    let mut context = ContextBuilder::default().host_hooks(hooks).build().unwrap();
    interval::register(&mut context).unwrap();
    context
}

#[test]
fn set_timeout_basic() {
    let clock = Rc::new(TestClockHooks::default());
    let context = &mut create_context(clock.clone());

    run_test_actions_with(
        [
            TestAction::run(
                r#"
            called = false;
            setTimeout(() => { called = true; });
        "#,
            ),
            TestAction::inspect_context(|ctx| {
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(false));

                ctx.run_jobs().unwrap();

                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn set_timeout_cancel() {
    let clock = Rc::new(TestClockHooks::default());
    let context = &mut create_context(clock.clone());
    let clock1 = clock.clone();
    let clock2 = clock.clone();

    run_test_actions_with(
        [
            TestAction::run(
                r#"
                called = false;
                id = setTimeout(() => { called = true; }, 100);
            "#,
            ),
            TestAction::inspect_context(|ctx| {
                let clock = clock1;
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(false));
                ctx.run_jobs().unwrap();

                clock.forward(50);
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(false));
                ctx.run_jobs().unwrap();
            }),
            TestAction::run("clearTimeout(id);"),
            TestAction::inspect_context(|ctx| {
                let clock = clock2;
                clock.forward(100);
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                // Should still be false, as it was cancelled.
                assert_eq!(called.as_boolean(), Some(false));
            }),
        ],
        context,
    );
}

#[test]
fn set_timeout_delay() {
    let clock = Rc::new(TestClockHooks::default());
    let context = &mut create_context(clock.clone());

    run_test_actions_with(
        [
            TestAction::run(
                r#"
            called = false;
            setTimeout(() => { called = true; }, 100);
        "#,
            ),
            TestAction::inspect_context(move |ctx| {
                // As long as the clock isn't updated, `called` will always be false.
                for _ in 0..5 {
                    let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                    assert_eq!(called.as_boolean(), Some(false));
                    ctx.run_jobs().unwrap();
                }

                // Move forward 50 milliseconds, `called` should still be false.
                clock.forward(50);
                ctx.run_jobs().unwrap();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(false));

                clock.forward(50);
                ctx.run_jobs().unwrap();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn set_interval_delay() {
    let clock = Rc::new(TestClockHooks::default());
    let context = &mut create_context(clock.clone());
    let clock1 = clock.clone(); // For the first test.
    let clock2 = clock.clone(); // For the first test.

    run_test_actions_with(
        [
            TestAction::run(
                r#"
                called = 0;
                id = setInterval(() => { called++; }, 100);
            "#,
            ),
            TestAction::inspect_context(|ctx| {
                let clock = clock1;
                // As long as the clock isn't updated, `called` will always be 0.
                for _ in 0..5 {
                    let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                    assert_eq!(called.as_i32(), Some(0));
                    ctx.run_jobs().unwrap();
                }

                // Move forward 50 milliseconds.
                clock.forward(50);
                ctx.run_jobs().unwrap();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(0));

                // Move forward 50 milliseconds.
                clock.forward(50);
                ctx.run_jobs().unwrap();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(1));

                // Move forward 50 milliseconds.
                clock.forward(50);
                ctx.run_jobs().unwrap();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(1));

                // Move forward 50 milliseconds.
                clock.forward(50);
                ctx.run_jobs().unwrap();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(2));

                // Move forward 500 milliseconds, should only be called once.
                clock.forward(500);
                ctx.run_jobs().unwrap();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(3));
            }),
            // Cancel
            TestAction::run("clearInterval(id);"),
            TestAction::inspect_context(move |ctx| {
                let clock = clock2;
                // Doesn't matter how long, this should not be called ever again.
                clock.forward(500);
                ctx.run_jobs().unwrap();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(3));

                clock.forward(500);
                ctx.run_jobs().unwrap();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(3));
            }),
        ],
        context,
    );
}
