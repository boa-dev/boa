use boa_macros::js_str;
use indoc::indoc;

use crate::{JsNativeErrorKind, TestAction, run_test_actions};

#[test]
// https://github.com/boa-dev/boa/issues/2317
fn fun_block_eval_2317() {
    run_test_actions([
        TestAction::assert_eq(
            indoc! {r#"
                (function(y){
                    {
                        eval("var x = 'inner';");
                    }
                    return y + x;
                })("arg");
            "#},
            js_str!("arginner"),
        ),
        TestAction::assert_eq(
            indoc! {r#"
                (function(y = "default"){
                    {
                        eval("var x = 'inner';");
                    }
                    return y + x;
                })();
            "#},
            js_str!("defaultinner"),
        ),
    ]);
}

#[test]
// https://github.com/boa-dev/boa/issues/2719
fn with_env_not_panic() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            with({ p1:1,  }) {k[oa>>2]=d;}
            {
            let a12345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890 = 1,
                b = "";
            }
        "#},
        JsNativeErrorKind::Reference,
        "k is not defined",
    )]);
}

#[test]
// https://github.com/boa-dev/boa/issues/4350
fn indirect_eval_function_var_binding_4350() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            var t = [];

            var s1 = `
            function core() { t.push(1) }

            core.prototype.a = function () { t.push(2) }
            core.prototype.b = function () { t.push(3) }
            `;
            var s2 = `
            function core() { t.push(1) }

            core.prototype.a = function () { t.push(2) }
            core.prototype.b = function () { t.push(3) }
            var core = new core();
            `;
            var s3 = `
            function core() { t.push(1) }
            var core = new core();
            `;

            function run_ctx(s) {
                (1,eval)(s);
            }

            function test() {
                run_ctx(s1);
                var core1 = new core();

                run_ctx(s2);
                var core2 = core;

                run_ctx(s3);
                var core3 = core;
                return [core1, core2, core3].toString();
            }

            test();
        "#},
        js_str!("[object Object],[object Object],[object Object]"),
    )]);
}

#[test]
// https://github.com/boa-dev/boa/issues/5333
fn eval_created_bindings_can_be_deleted_5333() {
    run_test_actions([
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    var initial = null;
                    var deleted = null;
                    var postDeletion;
                    eval('initial = x; deleted = delete x; postDeletion = function() { x; }; var x;');
                    try {
                        postDeletion();
                        return 'no throw';
                    } catch (e) {
                        return String(initial) + ':' + String(deleted) + ':' + e.name;
                    }
                }());
            "#},
            js_str!("undefined:true:ReferenceError"),
        ),
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    var initial;
                    var deleted = null;
                    var postDeletion;
                    eval('initial = f; deleted = delete f; postDeletion = function() { f; }; function f() { return 33; }');
                    try {
                        postDeletion();
                        return 'no throw';
                    } catch (e) {
                        return typeof initial + ':' + String(initial()) + ':' + String(deleted) + ':' + e.name;
                    }
                }());
            "#},
            js_str!("function:33:true:ReferenceError"),
        ),
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    delete globalThis.x;
                    eval('delete x; var x = 1;');
                    var result = typeof globalThis.x + ':' + String(globalThis.x);
                    delete globalThis.x;
                    return result;
                }());
            "#},
            js_str!("number:1"),
        ),
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    delete globalThis.x;
                    var result = eval('var x = delete x; x;');
                    var global = globalThis.x;
                    delete globalThis.x;
                    return String(result) + ':' + String(global);
                }());
            "#},
            js_str!("true:true"),
        ),
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    delete globalThis.x;
                    var x = 'outer';
                    var result = (function() {
                        return eval('var x = delete x; x;');
                    }());
                    var global = globalThis.x;
                    delete globalThis.x;
                    return String(result) + ':' + String(x) + ':' + String(global);
                }());
            "#},
            js_str!("true:true:undefined"),
        ),
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    delete globalThis.x;
                    eval('var x; delete x;');
                    eval('var x = 2;');
                    var result = String(x) + ':' + String(globalThis.x);
                    delete globalThis.x;
                    return result;
                }());
            "#},
            js_str!("2:undefined"),
        ),
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    delete globalThis.f;
                    eval('function f() {}; delete f;');
                    eval('function f() { return 2; }');
                    var result = String(f()) + ':' + String(globalThis.f);
                    delete globalThis.f;
                    return result;
                }());
            "#},
            js_str!("2:undefined"),
        ),
    ]);
}

#[test]
#[cfg(feature = "annex-b")]
fn eval_var_redeclares_catch_parameter() {
    // Annex B.3.4: var and function declarations introduced by a direct eval may redeclare
    // the parameter of the enclosing catch clause.
    run_test_actions([
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    try {
                        throw 1;
                    } catch (err) {
                        eval('var err = 2;');
                        var inner = err;
                    }
                    return String(inner) + ':' + typeof err;
                }());
            "#},
            js_str!("2:undefined"),
        ),
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    try {
                        throw 1;
                    } catch (err) {
                        eval('function err() {}');
                        var inner = typeof err;
                    }
                    return inner + ':' + typeof err;
                }());
            "#},
            js_str!("number:function"),
        ),
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    try {
                        throw 1;
                    } catch (err) {
                        eval('for (var err of []) {}');
                        return err;
                    }
                }());
            "#},
            1,
        ),
    ]);
}

#[test]
fn eval_var_conflicts_with_lexical_binding() {
    // A lexical binding that is not a catch parameter still conflicts with the var declaration.
    run_test_actions([
        TestAction::assert_native_error(
            indoc! {r#"
                (function() {
                    let x;
                    {
                        eval('var x;');
                    }
                }());
            "#},
            JsNativeErrorKind::Syntax,
            "variable declaration x in eval function already exists as a lexical variable",
        ),
        TestAction::assert_native_error(
            indoc! {r#"
                (function() {
                    try {
                        throw 1;
                    } catch (err) {
                        {
                            let err;
                            eval('var err;');
                        }
                    }
                }());
            "#},
            JsNativeErrorKind::Syntax,
            "variable declaration err in eval function already exists as a lexical variable",
        ),
    ]);
}

#[test]
#[cfg(feature = "annex-b")]
fn eval_block_function_hoisting_respects_lexical_bindings() {
    // Annex B.3.2.3: a block-level function in eval code is not hoisted to the variable
    // environment when an enclosing scope already has a binding with the same name.
    run_test_actions([
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    {
                        let f = 123;
                        eval('{ function f() {} }');
                    }
                    return typeof f;
                }());
            "#},
            js_str!("undefined"),
        ),
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    {
                        eval('{ function f() {} }');
                    }
                    return typeof f;
                }());
            "#},
            js_str!("function"),
        ),
        // The parameter of a catch clause does not prevent the hoisting.
        TestAction::assert_eq(
            indoc! {r#"
                (function() {
                    try {
                        throw 1;
                    } catch (f) {
                        eval('{ function f() {} }');
                        var inner = typeof f;
                    }
                    return inner + ':' + typeof f;
                }());
            "#},
            js_str!("number:function"),
        ),
    ]);
}
