use indoc::indoc;

use crate::{TestAction, run_test_actions};

#[test]
fn iterator_close_in_continue_before_jobs() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
            var actual = [];

            var iter = {
                [Symbol.iterator]() {
                    return this;
                },
                next() {
                    actual.push("call next");
                    return {
                        done: false,
                    };
                },
                get return() {
                    actual.push("get return");
                    return function () {
                        actual.push("return call");
                        return {
                            done: true
                        }
                    }
                }
            };

            Promise.resolve(0)
                .then(() => actual.push("tick 1"))
                .then(() => actual.push("tick 2"));

            void async function f() {
                actual.push("async fn start");
                let count = 0;
                loop: while (count === 0) {
                    count++;
                    for (_ of iter) {
                        continue loop;
                    }
                }
                actual.push("async fn end");
            }();
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert(indoc! {r#"
            arrayEquals(
                actual,
                [
                    "async fn start",
                    "call next",
                    "get return",
                    "return call",
                    "async fn end",
                    "tick 1",
                    "tick 2",
                ]
            )
            "#}),
    ]);
}

#[test]
fn async_iterator_close_in_continue_is_awaited() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
            var actual = [];

            var asyncIter = {
                [Symbol.asyncIterator]() {
                    return this;
                },
                next() {
                    actual.push("async call next");
                    return {
                        done: false,
                    };
                },
                get return() {
                    actual.push("get async return");
                    return function () {
                        actual.push("async return call");
                        return {
                            done: true
                        };
                    }
                }
            };

            Promise.resolve(0)
                .then(() => actual.push("tick 1"))
                .then(() => actual.push("tick 2"))
                .then(() => actual.push("tick 3"));

            void async function f() {
                actual.push("async fn start");
                let count = 0;
                loop: while (count === 0) {
                    count++;
                    for await (__ of asyncIter) {
                        continue loop;
                    }
                }
                actual.push("async fn end");
            }();
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert(indoc! {r#"
            arrayEquals(
                actual,
                [
                    "async fn start",
                    "async call next",
                    "tick 1",
                    "get async return",
                    "async return call",
                    "tick 2",
                    "async fn end",
                    "tick 3"
                ]
            )
        "#}),
    ]);
}

#[test]
fn mixed_iterators_close_in_continue() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
            var actual = [];

            var iter = {
                [Symbol.iterator]() {
                    return this;
                },
                next() {
                    actual.push("call next");
                    return {
                        done: false,
                    };
                },
                get return() {
                    actual.push("get return");
                    return function () {
                        actual.push("return call");
                        return {
                            done: true
                        }
                    }
                }
            };

            var asyncIter = {
                [Symbol.asyncIterator]() {
                    return this;
                },
                next() {
                    actual.push("async call next");
                    return {
                        done: false,
                    };
                },
                get return() {
                    actual.push("get async return");
                    return function () {
                        actual.push("async return call");
                        return {
                            done: true
                        };
                    }
                }
            };

            Promise.resolve(0)
                .then(() => actual.push("tick 1"))
                .then(() => actual.push("tick 2"))
                .then(() => actual.push("tick 3"));

            void async function f() {
                actual.push("async fn start");
                let count = 0;
                loop: while (count === 0) {
                    count++;
                    for (_ of iter) {
                        for await (__ of asyncIter) {
                            continue loop;
                        }
                    }
                }
                actual.push("async fn end");
            }();
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert(indoc! {r#"
            arrayEquals(
                actual,
                [
                    "async fn start",
                    "call next",
                    "async call next",
                    "tick 1",
                    "get async return",
                    "async return call",
                    "tick 2",
                    "get return",
                    "return call",
                    "async fn end",
                    "tick 3",
                ]
            )
        "#}),
    ]);
}

#[test]
fn array_rest_iterator_first_result_done() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
            let calls = 0;
            const it = {
                [Symbol.iterator]() {
                    return this;
                },
                next() {
                    calls++;
                    return { done: true };
                }
            };
            const [x, ...y] = it;

            let assignCalls = 0;
            const it2 = {
                [Symbol.iterator]() {
                    return this;
                },
                next() {
                    assignCalls++;
                    return { done: true };
                }
            };
            let a, b;
            [a, ...b] = it2;
        "#}),
        TestAction::assert_eq("calls", 1),
        TestAction::assert("x === undefined"),
        TestAction::assert("arrayEquals(y, [])"),
        TestAction::assert_eq("assignCalls", 1),
        TestAction::assert("a === undefined"),
        TestAction::assert("arrayEquals(b, [])"),
    ]);
}

#[test]
fn array_rest_iterator_multiple_values() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
            let calls = 0;
            const values = [1, 2, 3];
            const it = {
                [Symbol.iterator]() {
                    return this;
                },
                next() {
                    calls++;
                    if (values.length > 0) {
                        return { value: values.shift(), done: false };
                    }
                    return { done: true };
                }
            };
            const [first, ...rest] = it;
        "#}),
        TestAction::assert_eq("first", 1),
        TestAction::assert("arrayEquals(rest, [2, 3])"),
        TestAction::assert_eq("calls", 4),
    ]);
}

#[test]
fn array_rest_iterator_next_throws() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let calls = 0;
            const it = {
                [Symbol.iterator]() {
                    return this;
                },
                next() {
                    calls++;
                    if (calls === 2) {
                        throw new Error("iterator next error");
                    }
                    return { value: calls, done: false };
                }
            };
            let caught = false;
            let errorMsg = "";
            try {
                const [first, ...rest] = it;
            } catch (err) {
                caught = true;
                errorMsg = err.message;
            }
        "#}),
        TestAction::assert("caught"),
        TestAction::assert_eq("calls", 2),
        TestAction::assert("errorMsg === 'iterator next error'"),
    ]);
}

#[test]
fn array_rest_normal_destructuring() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
            const [a, b, ...rest] = [1, 2, 3, 4, 5];
            const [...all] = [10, 20];
            const [x, ...empty] = [42];
            const [...emptyAll] = [];
        "#}),
        TestAction::assert_eq("a", 1),
        TestAction::assert_eq("b", 2),
        TestAction::assert("arrayEquals(rest, [3, 4, 5])"),
        TestAction::assert("arrayEquals(all, [10, 20])"),
        TestAction::assert_eq("x", 42),
        TestAction::assert("arrayEquals(empty, [])"),
        TestAction::assert("arrayEquals(emptyAll, [])"),
    ]);
}

#[test]
fn array_rest_iterator_already_done_before_rest() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
            let calls = 0;
            const it = {
                [Symbol.iterator]() {
                    return this;
                },
                next() {
                    calls++;
                    if (calls === 1) {
                        return { value: "first", done: false };
                    }
                    return { done: true };
                }
            };
            const [a, b, ...rest] = it;
        "#}),
        TestAction::assert_eq("calls", 2),
        TestAction::assert("a === 'first'"),
        TestAction::assert("b === undefined"),
        TestAction::assert("arrayEquals(rest, [])"),
    ]);
}
