use crate::interval;
use crate::test::{run_test_actions_with, TestAction};
use boa_engine::{js_str, Context, Finalize, Trace};
use boa_gc::{Gc, GcRefCell};

#[derive(Debug, Clone, Trace, Finalize)]
struct TestClock(Gc<GcRefCell<u64>>);

impl interval::Clock for TestClock {
    fn now(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH + std::time::Duration::from_millis(*self.0.borrow())
    }
}

impl TestClock {
    fn new(time: u64) -> Self {
        Self(Gc::new(GcRefCell::new(time)))
    }

    fn forward(&self, time: u64) {
        *self.0.borrow_mut() += time;
    }
}

#[test]
fn set_timeout_basic() {
    let context = &mut Context::default();
    interval::register(context).unwrap();

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
                ctx.run_jobs();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn set_timeout_cancel() {
    let context = &mut Context::default();
    let clock = TestClock::new(0);
    let clock1 = clock.clone();
    let clock2 = clock.clone();
    interval::register_with_clock(context, clock).unwrap();

    run_test_actions_with(
        [
            TestAction::run(
                r#"
                called = false;
                id = setTimeout(() => { called = true; }, 100);
            "#,
            ),
            TestAction::inspect_context(move |ctx| {
                let clock = clock1;
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(false));
                ctx.run_jobs();

                clock.forward(50);
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(false));
                ctx.run_jobs();
            }),
            TestAction::run("clearTimeout(id);"),
            TestAction::inspect_context(move |ctx| {
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
    let context = &mut Context::default();
    let clock = TestClock::new(0);
    interval::register_with_clock(context, clock.clone()).unwrap();

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
                    ctx.run_jobs();
                }

                // Move forward 50 milliseconds, `called` should still be false.
                clock.forward(50);
                ctx.run_jobs();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(false));

                clock.forward(50);
                ctx.run_jobs();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn set_interval_delay() {
    let context = &mut Context::default();
    let clock = TestClock::new(0);
    let clock1 = clock.clone(); // For the first test.
    let clock2 = clock.clone(); // For the first test.
    interval::register_with_clock(context, clock.clone()).unwrap();

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
                    ctx.run_jobs();
                }

                // Move forward 50 milliseconds.
                clock.forward(50);
                ctx.run_jobs();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(0));

                // Move forward 50 milliseconds.
                clock.forward(50);
                ctx.run_jobs();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(1));

                // Move forward 50 milliseconds.
                clock.forward(50);
                ctx.run_jobs();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(1));

                // Move forward 50 milliseconds.
                clock.forward(50);
                ctx.run_jobs();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(2));

                // Move forward 500 milliseconds, should only be called once.
                clock.forward(500);
                ctx.run_jobs();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(3));
            }),
            // Cancel
            TestAction::run("clearInterval(id);"),
            TestAction::inspect_context(move |ctx| {
                let clock = clock2;
                // Doesn't matter how long, this should not be called ever again.
                clock.forward(500);
                ctx.run_jobs();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(3));

                clock.forward(500);
                ctx.run_jobs();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_i32(), Some(3));
            }),
        ],
        context,
    );
}
