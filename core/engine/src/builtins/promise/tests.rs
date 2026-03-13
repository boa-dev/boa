use std::{cell::Cell, rc::Rc};

use crate::{
    Context, JsObject, TestAction, run_test_actions, run_test_actions_with,
    builtins::Promise,
    builtins::promise::OperationType,
    context::{ContextBuilder, HostHooks},
};
use indoc::indoc;

#[test]
fn promise() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                    let count = 0;
                    const promise = new Promise((resolve, reject) => {
                        count += 1;
                        resolve(undefined);
                    }).then((_) => (count += 1));
                    count += 1;
                "#}),
        TestAction::assert_eq("count", 2),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("count", 3),
    ]);
}

#[test]
fn promise_all_resolves_values() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var values = [];
            var p = Promise.all([Promise.resolve(1), Promise.resolve(2)]);
            p.then(v => { values = v; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("values.length", 2),
        TestAction::assert_eq("values[0]", 1),
        TestAction::assert_eq("values[1]", 2),
    ]);
}

#[test]
fn promise_all_rejects() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var err = null;
            var p = Promise.all([Promise.resolve(1), Promise.reject(2)]);
            p.catch(e => { err = e; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("err", 2),
    ]);
}

#[test]
fn promise_any_resolves_first_success() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var val = null;
            var p = Promise.any([Promise.reject(1), Promise.resolve(2)]);
            p.then(v => { val = v; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("val", 2),
    ]);
}

#[test]
fn promise_all_settled_resolves_results() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var values = [];
            Promise.allSettled([
                Promise.resolve(1),
                Promise.reject(2)
            ]).then(results => {
                values = [
                    results[0].status,
                    results[0].value,
                    results[1].status,
                    results[1].reason
                ];
            });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("values[0]", crate::js_string!("fulfilled")),
        TestAction::assert_eq("values[1]", 1),
        TestAction::assert_eq("values[2]", crate::js_string!("rejected")),
        TestAction::assert_eq("values[3]", 2),
    ]);
}

#[test]
fn promise_race_resolves_first() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var val = null;
            Promise.race([
                Promise.resolve(10),
                Promise.resolve(20)
            ]).then(v => { val = v; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("val", 10),
    ]);
}

/// Regression test for <https://github.com/boa-dev/boa/issues/XXXX>
///
/// `PerformPromiseThen` (ECMA-262 §27.2.5.4.1 step 12) must set
/// `promise.[[PromiseIsHandled]]` unconditionally, regardless of the
/// promise's current state.  When a `.catch()` is attached while the
/// promise is still *pending*, the `handled` flag must be `true` before
/// the promise settles so that `RejectPromise` does not fire a spurious
/// `HostPromiseRejectionTracker("reject")` event.
#[test]
fn promise_is_handled_set_on_pending_promise() {
    // Custom hooks that count how many times the tracker fires for
    // the "reject" operation (= unhandled rejection).
    struct TrackingHooks {
        reject_count: Rc<Cell<u32>>,
    }

    impl HostHooks for TrackingHooks {
        fn promise_rejection_tracker(
            &self,
            _promise: &JsObject<Promise>,
            operation: OperationType,
            _context: &mut Context,
        ) {
            if operation == OperationType::Reject {
                self.reject_count.set(self.reject_count.get() + 1);
            }
        }
    }

    let reject_count = Rc::new(Cell::new(0u32));
    let hooks = Rc::new(TrackingHooks {
        reject_count: Rc::clone(&reject_count),
    });

    let mut context = ContextBuilder::new()
        .host_hooks(hooks)
        .build()
        .expect("context builds");

    // Attach .catch() while the promise is still pending, then reject it.
    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                var caught = false;
                var rejectFn;
                var p = new Promise((_resolve, reject) => { rejectFn = reject; });
                p.catch(() => { caught = true; });
                rejectFn(42);
            "#}),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            // The .catch() handler must have fired.
            TestAction::assert("caught"),
        ],
        &mut context,
    );

    // No "unhandled rejection" event should have been fired because a
    // handler was registered before the promise was rejected.
    assert_eq!(
        reject_count.get(),
        0,
        "HostPromiseRejectionTracker(\"reject\") must not fire when a handler was already attached"
    );
}
