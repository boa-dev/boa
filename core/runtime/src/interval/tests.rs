use crate::interval;
use crate::test::{TestAction, run_test_actions_with};
use boa_engine::context::time::FixedClock;
use boa_engine::context::{Clock, ContextBuilder};
use boa_engine::job::{JobExecutor, SimpleJobExecutor};
use boa_engine::{Context, js_str};
use futures_lite::future;
use indoc::indoc;
use std::cell::RefCell;
use std::pin;
use std::rc::Rc;

fn create_context(clock: Rc<impl Clock + 'static>) -> Context {
    let mut context = ContextBuilder::default().clock(clock).build().unwrap();
    interval::register(&mut context).unwrap();
    context
}

#[test]
fn set_timeout_basic() {
    let clock = Rc::new(FixedClock::default());
    let context = &mut create_context(clock.clone());

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                called = false;
                setTimeout(() => { called = true; });
            "#}),
            TestAction::inspect_context(move |ctx| {
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                assert_eq!(called.as_boolean(), Some(false));

                clock.forward(1);
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
    let clock = Rc::new(FixedClock::default());
    let context = &mut create_context(clock.clone());
    let clock1 = clock.clone();
    let clock2 = clock.clone();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                called = false;
                id = setTimeout(() => { called = true; }, 100);
            "#}),
            TestAction::inspect_context_async(async |ctx| {
                let clock = clock1;

                let job_executor = ctx.downcast_job_executor::<SimpleJobExecutor>().unwrap();
                let called = ctx.global_object().get(js_str!("called"), ctx).unwrap();
                let ctx = &RefCell::new(ctx);
                let mut event_loop = pin::pin!(job_executor.run_jobs_async(ctx));

                assert_eq!(called.as_boolean(), Some(false));

                assert!(future::poll_once(event_loop.as_mut()).await.is_none());
                clock.forward(50);
                assert!(future::poll_once(event_loop.as_mut()).await.is_none());

                let global_object = ctx.borrow().global_object();
                let called = global_object
                    .get(js_str!("called"), &mut ctx.borrow_mut())
                    .unwrap();
                assert_eq!(called.as_boolean(), Some(false));
                assert!(future::poll_once(event_loop).await.is_none());
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
    let clock = Rc::new(FixedClock::default());
    let context = &mut create_context(clock.clone());

    run_test_actions_with(
        [
            TestAction::run(
                r#"
            called = false;
            setTimeout(() => { called = true; }, 100);
        "#,
            ),
            TestAction::inspect_context_async(async move |ctx| {
                let global_object = ctx.global_object();
                let job_executor = ctx.downcast_job_executor::<SimpleJobExecutor>().unwrap();
                let ctx = &RefCell::new(ctx);
                let mut event_loop = pin::pin!(job_executor.run_jobs_async(ctx));

                // As long as the clock isn't updated, `called` will always be false.
                for _ in 0..5 {
                    let called = global_object
                        .get(js_str!("called"), &mut ctx.borrow_mut())
                        .unwrap();
                    assert_eq!(called.as_boolean(), Some(false));
                    assert!(future::poll_once(event_loop.as_mut()).await.is_none());
                }

                // Move forward 50 milliseconds, `called` should still be false.
                clock.forward(50);
                assert!(future::poll_once(event_loop.as_mut()).await.is_none());
                let called = global_object
                    .get(js_str!("called"), &mut ctx.borrow_mut())
                    .unwrap();
                assert_eq!(called.as_boolean(), Some(false));

                clock.forward(51);
                event_loop.await.unwrap();
                let called = global_object
                    .get(js_str!("called"), &mut ctx.borrow_mut())
                    .unwrap();
                assert_eq!(called.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn set_interval_delay() {
    let clock = Rc::new(FixedClock::default());
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
            TestAction::inspect_context_async(async |ctx| {
                let clock = clock1;
                let global_object = ctx.global_object();
                let job_executor = ctx.downcast_job_executor::<SimpleJobExecutor>().unwrap();
                let ctx = &RefCell::new(ctx);
                let mut event_loop = pin::pin!(job_executor.run_jobs_async(ctx));

                // As long as the clock isn't updated, `called` will always be false.
                for _ in 0..5 {
                    let called = global_object
                        .get(js_str!("called"), &mut ctx.borrow_mut())
                        .unwrap();
                    assert_eq!(called.as_i32(), Some(0));
                    assert!(future::poll_once(event_loop.as_mut()).await.is_none());
                }

                // Move forward 50 milliseconds.
                clock.forward(50);
                assert!(future::poll_once(event_loop.as_mut()).await.is_none());
                let called = global_object
                    .get(js_str!("called"), &mut ctx.borrow_mut())
                    .unwrap();
                assert_eq!(called.as_i32(), Some(0));

                // Move forward 51 milliseconds.
                clock.forward(51);
                assert!(future::poll_once(event_loop.as_mut()).await.is_none());
                let called = global_object
                    .get(js_str!("called"), &mut ctx.borrow_mut())
                    .unwrap();
                assert_eq!(called.as_i32(), Some(1));

                // Move forward 50 milliseconds.
                clock.forward(50);
                assert!(future::poll_once(event_loop.as_mut()).await.is_none());
                let called = global_object
                    .get(js_str!("called"), &mut ctx.borrow_mut())
                    .unwrap();
                assert_eq!(called.as_i32(), Some(1));

                // Move forward 51 milliseconds.
                clock.forward(51);
                assert!(future::poll_once(event_loop.as_mut()).await.is_none());
                let called = global_object
                    .get(js_str!("called"), &mut ctx.borrow_mut())
                    .unwrap();
                assert_eq!(called.as_i32(), Some(2));

                // Move forward 500 milliseconds, should only be called once.
                clock.forward(500);
                assert!(future::poll_once(event_loop.as_mut()).await.is_none());
                let called = global_object
                    .get(js_str!("called"), &mut ctx.borrow_mut())
                    .unwrap();
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
