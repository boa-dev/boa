use crate::builtins::promise::PromiseState;
use crate::error::RuntimeLimitError;
use crate::vm::CallFrame;
use crate::vm::call_frame::CallFrameLocation;
use crate::vm::source_info::SourcePath;
use crate::{
    Context, JsNativeError, JsNativeErrorKind, JsValue, Module, NativeFunction, TestAction,
    js_string, property::Attribute, run_test_actions, run_test_actions_with,
};
use boa_ast::Position;
use boa_macros::js_str;
use boa_parser::Source;
use indoc::indoc;

#[test]
fn typeof_string() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            const a = "hello";
            typeof a;
        "#},
        js_str!("string"),
    )]);
}

#[test]
fn typeof_number() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let a = 1234;
            typeof a;
        "#},
        js_str!("number"),
    )]);
}

#[test]
fn basic_op() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            const a = 1;
            const b = 2;
            a + b
        "#},
        3,
    )]);
}

#[test]
fn position() {
    let context = &mut Context::default();
    context
        .register_global_callable(
            js_string!("check_stack"),
            2,
            NativeFunction::from_copy_closure(|_, _, context| {
                let frame = context.stack_trace().collect::<Vec<&CallFrame>>();

                assert_eq!(frame.len(), 4);
                assert_eq!(
                    frame[0].position(),
                    CallFrameLocation {
                        function_name: js_string!("myOtherFunction"),
                        path: SourcePath::None,
                        position: Some(Position::new(2, 16))
                    }
                );
                assert_eq!(
                    frame[1].position(),
                    CallFrameLocation {
                        function_name: js_string!("<eval>"),
                        path: SourcePath::Eval,
                        position: Some(Position::new(1, 16))
                    }
                );
                assert_eq!(
                    frame[2].position(),
                    CallFrameLocation {
                        function_name: js_string!("myFunction"),
                        path: SourcePath::None,
                        position: Some(Position::new(5, 9))
                    }
                );
                assert_eq!(
                    frame[3].position(),
                    CallFrameLocation {
                        function_name: js_string!("<main>"),
                        path: SourcePath::None,
                        position: Some(Position::new(8, 11))
                    }
                );
                Ok(JsValue::undefined())
            }),
        )
        .expect("Could not register function");
    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            const myOtherFunction = () => {
                check_stack();
            };
            function myFunction() {
                eval("myOtherFunction()");
            }

            myFunction();
        "#})],
        context,
    );
}

#[test]
fn try_catch_finally_from_init() {
    // the initialisation of the array here emits a PopOnReturnAdd op
    //
    // here we test that the stack is not popped more than intended due to multiple catches in the
    // same function, which could lead to VM stack corruption
    run_test_actions([TestAction::assert_opaque_error(
        indoc! {r#"
            try {
                [(() => {throw "h";})()];
            } catch (x) {
                throw "h";
            } finally {
            }
        "#},
        js_str!("h"),
    )]);
}

#[test]
fn multiple_catches() {
    // see explanation on `try_catch_finally_from_init`
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            try {
                try {
                    [(() => {throw "h";})()];
                } catch (x) {
                    throw "h";
                }
            } catch (y) {
            }
        "#},
        JsValue::undefined(),
    )]);
}

#[test]
fn use_last_expr_try_block() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            try {
                19;
                7.5;
                "Hello!";
            } catch (y) {
                14;
                "Bye!"
            }
        "#},
        js_str!("Hello!"),
    )]);
}

#[test]
fn use_last_expr_catch_block() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            try {
                throw Error("generic error");
                19;
                7.5;
            } catch (y) {
                14;
                "Hello!";
            }
        "#},
        js_str!("Hello!"),
    )]);
}

#[test]
fn no_use_last_expr_finally_block() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            try {
            } catch (y) {
            } finally {
                "Unused";
            }
        "#},
        JsValue::undefined(),
    )]);
}

#[test]
fn finally_block_binding_env() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let buf = "Hey hey";
            try {
            } catch (y) {
            } finally {
                let x = " people";
                buf += x;
            }
            buf
        "#},
        js_str!("Hey hey people"),
    )]);
}

#[test]
fn run_super_method_in_object() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let proto = {
                m() { return "super"; }
            };
            let obj = {
                v() { return super.m(); }
            };
            Object.setPrototypeOf(obj, proto);
            obj.v();
        "#},
        js_str!("super"),
    )]);
}

#[test]
fn get_reference_by_super() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            var fromA, fromB;
            var A = { fromA: 'a', fromB: 'a' };
            var B = { fromB: 'b' };
            Object.setPrototypeOf(B, A);
            var obj = {
                fromA: 'c',
                fromB: 'c',
                method() {
                    fromA = (() => { return super.fromA; })();
                    fromB = (() => { return super.fromB; })();
                }
            };
            Object.setPrototypeOf(obj, B);
            obj.method();
            fromA + fromB
        "#},
        js_str!("ab"),
    )]);
}

#[test]
fn super_call_constructor_null() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            class A extends Object {
                constructor() {
                    Object.setPrototypeOf(A, null);
                    super(A);
                }
            }
            new A();
        "#},
        JsNativeErrorKind::Type,
        "super constructor object must be constructor",
    )]);
}

#[test]
fn super_call_get_constructor_before_arguments_execution() {
    run_test_actions([TestAction::assert(indoc! {r#"
        class A extends Object {
            constructor() {
                super(Object.setPrototypeOf(A, null));
            }
        }
        new A() instanceof A;
    "#})]);
}

#[test]
fn order_of_execution_in_assignment() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                let i = 0;
                let array = [[]];

                array[i++][i++] = i++;
            "#}),
        TestAction::assert_eq("i", 3),
        TestAction::assert_eq("array.length", 1),
        TestAction::assert_eq("array[0].length", 2),
    ]);
}

#[test]
fn order_of_execution_in_assignment_with_comma_expressions() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let result = "";
            function f(i) {
                result += i;
            }
            let a = [[]];
            (f(1), a)[(f(2), 0)][(f(3), 0)] = (f(4), 123);
            result
        "#},
        js_str!("1234"),
    )]);
}

#[test]
fn loop_runtime_limit() {
    run_test_actions([
        TestAction::assert_eq(
            indoc! {r#"
                for (let i = 0; i < 20; ++i) { }
            "#},
            JsValue::undefined(),
        ),
        TestAction::inspect_context(|context| {
            context.runtime_limits_mut().set_loop_iteration_limit(10);
        }),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                for (let i = 0; i < 20; ++i) { }
            "#},
            RuntimeLimitError::LoopIteration,
        ),
        TestAction::assert_eq(
            indoc! {r#"
                for (let i = 0; i < 10; ++i) { }
            "#},
            JsValue::undefined(),
        ),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                while (1) { }
            "#},
            RuntimeLimitError::LoopIteration,
        ),
    ]);
}

/// Test that the loop iteration limit allows exactly `limit` body executions
/// and that all loop kinds agree on the count.
///
/// See: <https://github.com/boa-dev/boa/issues/5461>
#[test]
fn loop_iteration_limit_counts_body_executions() {
    run_test_actions([
        TestAction::inspect_context(|context| {
            context.runtime_limits_mut().set_loop_iteration_limit(10);
        }),
        // Loops that run exactly `limit` iterations must complete without error.
        TestAction::assert_eq("var a = 0; for (let i = 0; i < 10; ++i) { a++; } a", 10),
        TestAction::assert_eq("var b = 0; while (b < 10) { b++; } b", 10),
        TestAction::assert_eq("var c = 0; do { c++; } while (c < 10); c", 10),
        TestAction::assert_eq(
            "var d = 0; for (const x of [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]) { d++; } d",
            10,
        ),
        TestAction::assert_eq(
            indoc! {r#"
                var e = 0;
                var o = { k0: 0, k1: 0, k2: 0, k3: 0, k4: 0, k5: 0, k6: 0, k7: 0, k8: 0, k9: 0 };
                for (const k in o) { e++; }
                e
            "#},
            10,
        ),
        // Loops that would exceed the limit must error after exactly `limit`
        // body executions, and `for` and `while` loops must agree.
        TestAction::assert_runtime_limit_error(
            "var forCount = 0; for (let i = 0; i < 1000; ++i) { forCount++; }",
            RuntimeLimitError::LoopIteration,
        ),
        TestAction::assert_eq("forCount", 10),
        TestAction::assert_runtime_limit_error(
            "var whileCount = 0; while (true) { whileCount++; }",
            RuntimeLimitError::LoopIteration,
        ),
        TestAction::assert_eq("whileCount", 10),
        TestAction::assert_runtime_limit_error(
            "var doWhileCount = 0; do { doWhileCount++; } while (true);",
            RuntimeLimitError::LoopIteration,
        ),
        TestAction::assert_eq("doWhileCount", 10),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                var forOfCount = 0;
                var infinite = {
                    [Symbol.iterator]() {
                        return { next() { return { done: false, value: 1 }; } };
                    }
                };
                for (const x of infinite) { forOfCount++; }
            "#},
            RuntimeLimitError::LoopIteration,
        ),
        TestAction::assert_eq("forOfCount", 10),
    ]);
}

#[test]
fn recursion_runtime_limit() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            function factorial(n) {
                if (n == 0) {
                    return 1;
                }

                return n * factorial(n - 1);
            }
        "#}),
        TestAction::assert_eq("factorial(8)", JsValue::new(40_320)),
        TestAction::assert_eq("factorial(11)", JsValue::new(39_916_800)),
        TestAction::inspect_context(|context| {
            context.runtime_limits_mut().set_recursion_limit(10);
        }),
        TestAction::assert_runtime_limit_error("factorial(11)", RuntimeLimitError::Recursion),
        TestAction::assert_eq("factorial(8)", JsValue::new(40_320)),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                function x() {
                    x()
                }

                x()
            "#},
            RuntimeLimitError::Recursion,
        ),
    ]);
}

#[test]
fn arguments_object_constructor_valid_index() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let args;
            function F(a = 1) {
                args = arguments;
            }
            new F();
            typeof args
        "#},
        js_str!("object"),
    )]);
}

#[test]
fn empty_return_values() {
    run_test_actions([
        TestAction::run(indoc! {r#"do {{}} while (false);"#}),
        TestAction::run(indoc! {r#"do try {{}} catch {} while (false);"#}),
        TestAction::run(indoc! {r#"do {} while (false);"#}),
        TestAction::run(indoc! {r#"do try {{}{}} catch {} while (false);"#}),
        TestAction::run(indoc! {r#"do {{}{}} while (false);"#}),
        TestAction::run(indoc! {r#"do {;{}} while (false);"#}),
        TestAction::run(indoc! {r#"do {e: {}} while (false);"#}),
        TestAction::run(indoc! {r#"do {e: ;} while (false);"#}),
        TestAction::run(indoc! {r#"do { break } while (false);"#}),
        TestAction::run(indoc! {r#"while (true) a: break"#}),
        TestAction::run(indoc! {r#"while (true) a: {"a"; break};"#}),
        TestAction::run(indoc! {r#"do {"a";{}} while (false);"#}),
        TestAction::run(indoc! {r#"
            switch (false) {
                default: {}
            }
        "#}),
        TestAction::run(indoc! {r#"
            switch (false) {
                default: {}{}
            }
        "#}),
        TestAction::run(indoc! {r#"
            switch (false) {
                default: ;{}{}
            }
        "#}),
    ]);
}

#[test]
fn truncate_environments_on_non_caught_native_error() {
    let source = "with (new Proxy({}, {has: p => false})) {a}";
    run_test_actions([
        TestAction::assert_native_error(source, JsNativeErrorKind::Reference, "a is not defined"),
        TestAction::assert_native_error(source, JsNativeErrorKind::Reference, "a is not defined"),
    ]);
}

#[test]
fn super_construction_with_parameter_expression() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class Person {
                constructor(name) {
                    this.name = name;
                }
            }

            class Student extends Person {
                constructor(name = 'unknown') {
                    super(name);
                }
            }
        "#}),
        TestAction::assert_eq("new Student().name", js_str!("unknown")),
        TestAction::assert_eq("new Student('Jack').name", js_str!("Jack")),
    ]);
}

#[test]
fn cross_context_function_call() {
    let context1 = &mut Context::default();
    let result = context1.eval(Source::from_bytes(indoc! {r"
        var global = 100;

        (function x() {
            return global;
        })
    "}));

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_callable());

    let context2 = &mut Context::default();

    context2
        .register_global_property(js_string!("func"), result, Attribute::all())
        .unwrap();

    let result = context2.eval(Source::from_bytes("func()"));

    assert_eq!(result, Ok(JsValue::new(100)));
}

// See: https://github.com/boa-dev/boa/issues/1848
#[test]
fn long_object_chain_gc_trace_stack_overflow() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let old = {};
            for (let i = 0; i < 100000; i++) {
                old = { old };
            }
        "#}),
        TestAction::inspect_context(|_| boa_gc::force_collect()),
    ]);
}

// See: https://github.com/boa-dev/boa/issues/4515
#[test]
fn recursion_in_async_gen_throws_uncatchable_error() {
    run_test_actions([
        TestAction::inspect_context(|context| {
            context.runtime_limits_mut().set_recursion_limit(128);
        }),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                async function* f() {}
                f().return({
                  get then() {
                    this.then;
                  },
                });
            "#},
            RuntimeLimitError::Recursion,
        ),
    ]);
}

#[test]
fn recursion_in_setter_throws_uncatchable_error() {
    run_test_actions([
        TestAction::inspect_context(|context| {
            context.runtime_limits_mut().set_recursion_limit(128);
        }),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                const obj = {
                  set x(value) {
                    this.x = value;
                  },
                };
                obj.x = 1;
            "#},
            RuntimeLimitError::Recursion,
        ),
    ]);
}

#[test]
fn with_object_environment_call_single_lookup_and_this() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let emptyHasCount = 0;
            const emptyProxy = new Proxy({}, {
                has(t, p) {
                    if (p === "Object") {
                        emptyHasCount++;
                    }
                    return Reflect.has(t, p);
                }
            });
            with (emptyProxy) {
                Object();
            }

            let hasCount = 0;
            let callThis = null;
            const target = {
                fn() {
                    callThis = this;
                }
            };
            const proxy = new Proxy(target, {
                has(t, p) {
                    if (p === "fn") {
                        hasCount++;
                    }
                    return Reflect.has(t, p);
                }
            });
            with (proxy) {
                fn();
            }
        "#}),
        TestAction::assert_eq("emptyHasCount", 1),
        TestAction::assert_eq("hasCount", 2),
        TestAction::assert("callThis === proxy"),
    ]);
}

#[test]
fn with_object_environment_binding_deleted_in_unscopables() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let unscopablesCalled = 0;
            const env = {
                binding: 42,
                get [Symbol.unscopables]() {
                    unscopablesCalled++;
                    delete env.binding;
                    return null;
                }
            };
            let sloppyResult = null;
            with (env) {
                sloppyResult = binding;
            }

            let strictThrew = false;
            const envStrict = {
                binding: 42,
                get [Symbol.unscopables]() {
                    delete envStrict.binding;
                    return null;
                }
            };
            with (envStrict) {
                try {
                    (function() {
                        "use strict";
                        return binding;
                    })();
                } catch (e) {
                    if (e instanceof ReferenceError) {
                        strictThrew = true;
                    }
                }
            }
        "#}),
        TestAction::assert_eq("unscopablesCalled", 1),
        TestAction::assert("sloppyResult === undefined"),
        TestAction::assert("strictThrew === true"),
    ]);
}

/// Creates a context with the functions that the value stack tests below use.
///
/// * `host(f)` is a native function that calls `f` from Rust and swallows the error it throws.
/// * `stackLength()` returns the length of the value stack.
fn value_stack_test_context() -> Context {
    let mut context = Context::default();
    context
        .register_global_callable(
            js_string!("host"),
            1,
            NativeFunction::from_fn_ptr(|_, args, context| {
                let f = args
                    .first()
                    .and_then(JsValue::as_callable)
                    .ok_or_else(JsNativeError::typ)?;
                assert!(f.call(&JsValue::undefined(), &[], context).is_err());
                Ok(js_string!("swallowed").into())
            }),
        )
        .unwrap();
    context
        .register_global_callable(
            js_string!("stackLength"),
            0,
            NativeFunction::from_fn_ptr(|_, _, context| Ok(context.vm.stack.len().into())),
        )
        .unwrap();
    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            function inner() { throw "nested"; }
            function outer() { inner(); }
            function pair(a, b) { return a + "|" + b; }
            function one() { return 1; }
        "#})],
        &mut context,
    );
    context
}

/// A call from a host function that throws, and whose error the host function handles, must leave
/// the frame that called the host function as it was, with the operands of its pending call in
/// place.
#[test]
fn caught_nested_throw_keeps_the_calling_frame_intact() {
    run_test_actions_with(
        [
            // The `Promise` constructor rejects with the error its executor throws.
            TestAction::assert_eq(
                r#"pair(new Promise(outer), "tail")"#,
                js_str!("[object Promise]|tail"),
            ),
            // Calling a class constructor without `new` throws before a frame is pushed.
            TestAction::assert_eq(
                r#"pair(new Promise(class {}), "tail")"#,
                js_str!("[object Promise]|tail"),
            ),
            TestAction::assert_eq(r#"pair(host(outer), "tail")"#, js_str!("swallowed|tail")),
            TestAction::assert_eq(r#"pair(host(inner), "tail")"#, js_str!("swallowed|tail")),
        ],
        &mut value_stack_test_context(),
    );
}

/// The frames that a throw unwinds must leave the value stack, wherever the unwinding stops: at a
/// handler in an outer frame, including one in a generator or an async function, whose stack
/// lives as long as they do, at a host call, or with an uncatchable error.
#[test]
fn nested_throws_do_not_exhaust_the_value_stack() {
    let context = &mut value_stack_test_context();
    // A small stack shows a leak of a few values per throw after a few dozen throws.
    context.runtime_limits_mut().set_stack_size_limit(256);

    run_test_actions_with(
        [
            // Caught by a handler in an outer frame.
            TestAction::assert_eq(
                indoc! {r#"
                    let caught = 0;
                    for (let i = 0; i < 500; i++) {
                        try { outer(); } catch { caught++; }
                    }
                    caught
                "#},
                500,
            ),
            // Caught by a handler in a generator that is resumed after each catch.
            TestAction::assert_eq(
                indoc! {r#"
                    let generatorCaught = 0;
                    function* catcher() {
                        for (;;) {
                            try { outer(); } catch { generatorCaught++; }
                            yield;
                        }
                    }
                    const catching = catcher();
                    for (let i = 0; i < 500; i++) { catching.next(); }
                    generatorCaught
                "#},
                500,
            ),
            // Caught by a handler in an async function that is resumed after each `await`.
            TestAction::run(indoc! {r#"
                let asyncCaught = 0;
                let settled = false;
                (async () => {
                    for (let i = 0; i < 500; i++) {
                        try { await null; outer(); } catch { asyncCaught++; }
                    }
                })().then(() => { settled = true; });
            "#}),
            TestAction::inspect_context(|context| context.run_jobs().unwrap()),
            TestAction::assert_eq("asyncCaught", 500),
            TestAction::assert("settled"),
            // Swallowed by the host call that it unwinds to.
            TestAction::assert_eq(
                indoc! {r#"
                    let swallowed = 0;
                    for (let i = 0; i < 500; i++) {
                        if (host(outer) === "swallowed") { swallowed++; }
                    }
                    swallowed
                "#},
                500,
            ),
        ],
        context,
    );

    // Uncaught, out of a script and out of a call from Rust.
    let outer = context
        .global_object()
        .get(js_string!("outer"), context)
        .unwrap();
    let outer = outer.as_callable().unwrap();
    for _ in 0..500 {
        let error = context.eval(Source::from_bytes("outer()")).unwrap_err();
        assert_eq!(error.as_opaque(), Some(&js_str!("nested").into()));
        let error = outer.call(&JsValue::undefined(), &[], context).unwrap_err();
        assert_eq!(error.as_opaque(), Some(&js_str!("nested").into()));
    }

    // Uncatchable, raised in a nested frame and in the script's own frame.
    context.runtime_limits_mut().set_loop_iteration_limit(10);
    run_test_actions_with(
        std::iter::once(TestAction::run(
            "function spin() { while (true) {} } function spinner() { spin(); }",
        ))
        .chain((0..500).flat_map(|_| {
            [
                TestAction::assert_runtime_limit_error(
                    "spinner()",
                    RuntimeLimitError::LoopIteration,
                ),
                TestAction::assert_runtime_limit_error(
                    "while (true) {}",
                    RuntimeLimitError::LoopIteration,
                ),
            ]
        }))
        .chain(std::iter::once(TestAction::assert_eq("one()", 1))),
        context,
    );
}

/// These call, evaluation and resumption paths must restore the value stack after a throw.
#[test]
fn throws_leave_the_value_stack_as_they_found_it() {
    let context = &mut value_stack_test_context();
    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                // How many values calling `f`, which must throw, leaves on the value stack. A bound
                // native `f` makes the host call from this frame, with no JavaScript frame in
                // between.
                function leak(f) {
                    const before = stackLength();
                    let caught = false;
                    try { f(); } catch { caught = true; }
                    if (!caught) { throw new Error("expected an exception"); }
                    return stackLength() - before;
                }
                const started = (function* () { outer(); })();
                const resumed = (function* () { yield; outer(); })();
                resumed.next();
            "#}),
            // Unwound to a handler in an outer frame.
            TestAction::assert_eq("leak(outer)", 0),
            // Unwound to a host call: from a native function, from the `eval` built-in and from
            // a `JSON.parse` reviver.
            TestAction::assert_eq("leak(Array.prototype.map.bind([0], outer))", 0),
            TestAction::assert_eq("leak(Reflect.construct.bind(null, outer, []))", 0),
            TestAction::assert_eq(r#"leak(eval.bind(null, "outer()"))"#, 0),
            TestAction::assert_eq(r#"leak(JSON.parse.bind(null, "[0]", outer))"#, 0),
            // Thrown by a host call before it pushes a frame: a class constructor called without
            // `new`, and a field initializer.
            TestAction::assert_eq("leak(Reflect.apply.bind(null, class {}, undefined, []))", 0),
            TestAction::assert_eq(
                "leak(Reflect.construct.bind(null, class { x = outer(); }, []))",
                0,
            ),
            // Thrown while instantiating the declarations of an `eval`.
            TestAction::assert_eq(r#"leak(eval.bind(null, "function NaN() {}"))"#, 0),
            // Thrown out of a generator, when it starts and when it resumes.
            TestAction::assert_eq("leak(started.next.bind(started))", 0),
            TestAction::assert_eq("leak(resumed.next.bind(resumed))", 0),
        ],
        context,
    );

    // Thrown out of a script, while instantiating a script's declarations, and out of an async
    // function when it resumes.
    let before = context.vm.stack.len();
    run_test_actions_with(
        [
            TestAction::assert_opaque_error("outer()", js_str!("nested")),
            TestAction::assert_native_error(
                "function NaN() {}",
                JsNativeErrorKind::Type,
                "cannot declare global function",
            ),
            TestAction::run(indoc! {r#"
                var rejection;
                (async () => { await 0; outer(); })().catch((e) => { rejection = e; });
            "#}),
            TestAction::inspect_context(|context| context.run_jobs().unwrap()),
            TestAction::assert_eq("rejection", js_str!("nested")),
        ],
        context,
    );
    assert_eq!(context.vm.stack.len(), before);

    // Thrown out of a module's evaluation, synchronously and after an `await`. The stack is
    // measured around the evaluation only, after linking.
    for source in ["outer();", "await 0; outer();"] {
        let module = Module::parse(Source::from_bytes(source), None, context).unwrap();
        let promise = module.load(context);
        context.run_jobs().unwrap();
        assert_eq!(
            promise.state(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        module.link(context).unwrap();

        let before = context.vm.stack.len();
        let promise = module.evaluate(context).unwrap();
        context.run_jobs().unwrap();
        assert_eq!(
            promise.state(),
            PromiseState::Rejected(js_str!("nested").into()),
            "{source}"
        );
        assert_eq!(context.vm.stack.len(), before, "{source}");
    }
}
