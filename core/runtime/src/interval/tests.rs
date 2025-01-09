use crate::interval;
use crate::test::{run_test_actions_with, TestAction};
use boa_engine::{js_str, Context};

#[test]
fn set_timeout_basic() {
    let context = &mut Context::default();
    interval::register(context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(
                r#"
            called = false;
            id = setTimeout(() => { called = true; }, 0);
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
fn set_timeout_delay() {
    let context = &mut Context::default();
    interval::register(context).unwrap();

    run_test_actions_with(
        [
            TestAction::run(
                r#"
            called = false;
            id = setTimeout(() => { called = true; }, 1);
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
