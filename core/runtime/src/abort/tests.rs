use crate::test::{TestAction, run_test_actions};

#[test]
fn abort_controller_exists() {
    run_test_actions([
        TestAction::run("let ctrl = new AbortController()"),
        TestAction::run("let sig = ctrl.signal"),
    ]);
}

#[test]
fn signal_not_aborted_initially() {
    run_test_actions([TestAction::run(
        r"
        let ctrl = new AbortController();
        if (ctrl.signal.aborted !== false) {
            throw new Error('signal should not be aborted initially');
        }
        ",
    )]);
}

#[test]
fn abort_sets_aborted() {
    run_test_actions([TestAction::run(
        r"
        let ctrl = new AbortController();
        ctrl.abort();
        if (ctrl.signal.aborted !== true) {
            throw new Error('signal should be aborted after abort()');
        }
        ",
    )]);
}

#[test]
fn abort_with_custom_reason() {
    run_test_actions([TestAction::run(
        r#"
        let ctrl = new AbortController();
        ctrl.abort("custom reason");
        if (ctrl.signal.reason !== "custom reason") {
            throw new Error('reason should be "custom reason", got: ' + ctrl.signal.reason);
        }
        "#,
    )]);
}

#[test]
fn abort_default_reason() {
    run_test_actions([TestAction::run(
        r#"
        let ctrl = new AbortController();
        ctrl.abort();
        if (ctrl.signal.reason.name !== "AbortError") {
            throw new Error('default reason.name should be "AbortError", got: ' + ctrl.signal.reason.name);
        }
        "#,
    )]);
}

#[test]
fn throw_if_aborted_does_nothing_when_not_aborted() {
    run_test_actions([TestAction::run(
        r"
        let ctrl = new AbortController();
        ctrl.signal.throwIfAborted();
        ",
    )]);
}

#[test]
fn throw_if_aborted_throws_when_aborted() {
    run_test_actions([TestAction::run(
        r#"
        let ctrl = new AbortController();
        ctrl.abort("test error");
        let threw = false;
        try {
            ctrl.signal.throwIfAborted();
        } catch (e) {
            threw = true;
            if (e !== "test error") {
                throw new Error('expected "test error", got: ' + e);
            }
        }
        if (!threw) {
            throw new Error('throwIfAborted should have thrown');
        }
        "#,
    )]);
}

#[test]
fn add_event_listener_fires_on_abort() {
    run_test_actions([
        TestAction::run(
            r"
            let ctrl = new AbortController();
            let called = false;
            ctrl.signal.addEventListener('abort', function() {
                called = true;
            });
            ctrl.abort();
            ",
        ),
        TestAction::inspect_context(|ctx| {
            ctx.run_jobs().unwrap();
        }),
        TestAction::run(
            r"
            if (!called) {
                throw new Error('abort listener was not called');
            }
            ",
        ),
    ]);
}

#[test]
fn multiple_listeners() {
    run_test_actions([
        TestAction::run(
            r"
            let ctrl = new AbortController();
            let count = 0;
            ctrl.signal.addEventListener('abort', function() { count += 1; });
            ctrl.signal.addEventListener('abort', function() { count += 1; });
            ctrl.signal.addEventListener('abort', function() { count += 1; });
            ctrl.abort();
            ",
        ),
        TestAction::inspect_context(|ctx| {
            ctx.run_jobs().unwrap();
        }),
        TestAction::run(
            r"
            if (count !== 3) {
                throw new Error('expected 3 listeners called, got: ' + count);
            }
            ",
        ),
    ]);
}

#[test]
fn repeated_abort_is_idempotent() {
    run_test_actions([
        TestAction::run(
            r"
            let ctrl = new AbortController();
            let count = 0;
            ctrl.signal.addEventListener('abort', function() { count += 1; });
            ctrl.abort();
            ctrl.abort();
            ctrl.abort();
            ",
        ),
        TestAction::inspect_context(|ctx| {
            ctx.run_jobs().unwrap();
        }),
        TestAction::run(
            r"
            if (count !== 1) {
                throw new Error('listeners should fire only once, got: ' + count);
            }
            ",
        ),
    ]);
}

#[test]
fn signal_reuse_across_references() {
    run_test_actions([TestAction::run(
        r"
        let ctrl = new AbortController();
        let sig1 = ctrl.signal;
        let sig2 = ctrl.signal;
        ctrl.abort();
        if (sig1.aborted !== true || sig2.aborted !== true) {
            throw new Error('both signal references should be aborted');
        }
        ",
    )]);
}

#[test]
fn reason_undefined_when_not_aborted() {
    run_test_actions([TestAction::run(
        r"
        let ctrl = new AbortController();
        if (ctrl.signal.reason !== undefined) {
            throw new Error('reason should be undefined before abort, got: ' + ctrl.signal.reason);
        }
        ",
    )]);
}

#[test]
fn reason_abort_error_when_aborted_without_reason() {
    run_test_actions([TestAction::run(
        r#"
        let ctrl = new AbortController();
        ctrl.abort();
        if (!(ctrl.signal.reason instanceof Error)) {
            throw new Error('reason should be an instance of Error');
        }
        if (ctrl.signal.reason.name !== "AbortError") {
            throw new Error('reason.name should be "AbortError", got: ' + ctrl.signal.reason.name);
        }
        "#,
    )]);
}
