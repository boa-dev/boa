//! This module implements the global `eval` function.
//!
//! The `eval()` function evaluates JavaScript code represented as a string.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-eval-x
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval

use crate::{
    builtins::{BuiltIn, JsArgs},
    environments::DeclarativeEnvironment,
    error::JsNativeError,
    object::FunctionBuilder,
    property::Attribute,
    Context, JsResult, JsValue,
};
use boa_gc::Gc;
use boa_profiler::Profiler;
use rustc_hash::FxHashSet;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Eval;

impl BuiltIn for Eval {
    const NAME: &'static str = "eval";

    const ATTRIBUTE: Attribute = Attribute::CONFIGURABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::WRITABLE);

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let object = FunctionBuilder::native(context, Self::eval)
            .name("eval")
            .length(1)
            .constructor(false)
            .build();

        Some(object.into())
    }
}

impl Eval {
    /// `19.2.1 eval ( x )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-eval-x
    fn eval(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Return ? PerformEval(x, false, false).
        Self::perform_eval(args.get_or_undefined(0), false, false, context)
    }

    /// `19.2.1.1 PerformEval ( x, strictCaller, direct )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performeval
    pub(crate) fn perform_eval(
        x: &JsValue,
        direct: bool,
        mut strict: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        enum EnvStackAction {
            Truncate(usize),
            Restore(Vec<Gc<DeclarativeEnvironment>>),
        }

        // 1. Assert: If direct is false, then strictCaller is also false.
        debug_assert!(direct || !strict);

        // 2. If Type(x) is not String, return x.
        let x = if let Some(x) = x.as_string() {
            x.clone()
        } else {
            return Ok(x.clone());
        };

        // Because of implementation details the following code differs from the spec.
        // TODO: rework parser to take an iterator of `u32` unicode codepoints

        // Parse the script body and handle early errors (6 - 11)
        let body = match context.parse_eval(x.to_std_string_escaped().as_bytes(), direct, strict) {
            Ok(body) => body,
            Err(e) => return Err(JsNativeError::syntax().with_message(e.to_string()).into()),
        };

        strict |= body.strict();

        // Because our environment model does not map directly to the spec, this section looks very different.
        // 12 - 13 are implicit in the call of `Context::compile_with_new_declarative`.
        // 14 - 33 are in the following section, together with EvalDeclarationInstantiation.
        let action = if direct {
            // If the call to eval is direct, the code is executed in the current environment.

            // Poison the current environment, because it may contain new declarations after/during eval.
            if !strict {
                context.realm.environments.poison_current();
            }

            // Set the compile time environment to the current running environment and save the number of current environments.
            context.realm.compile_env = context.realm.environments.current_compile_environment();
            let environments_len = context.realm.environments.len();

            // Pop any added runtime environments that were not removed during the eval execution.
            EnvStackAction::Truncate(environments_len)
        } else {
            // If the call to eval is indirect, the code is executed in the global environment.

            // Poison all environments, because the global environment may contain new declarations after/during eval.
            if !strict {
                context.realm.environments.poison_all();
            }

            // Pop all environments before the eval execution.
            let environments = context.realm.environments.pop_to_global();
            context.realm.compile_env = context.realm.environments.current_compile_environment();

            // Restore all environments to the state from before the eval execution.
            EnvStackAction::Restore(environments)
        };

        // Only need to check on non-strict mode since strict mode automatically creates a function
        // environment for all eval calls.
        if !strict {
            // Error if any var declaration in the eval code already exists as a let/const declaration in the current running environment.
            let mut vars = FxHashSet::default();
            body.var_declared_names(&mut vars);
            if let Some(name) = context
                .realm
                .environments
                .has_lex_binding_until_function_environment(&vars)
            {
                let name = context.interner().resolve_expect(name.sym());
                let msg = format!("variable declaration {name} in eval function already exists as a lexical variable");
                return Err(JsNativeError::syntax().with_message(msg).into());
            }
        }

        // TODO: check if private identifiers inside `eval` are valid.

        // Compile and execute the eval statement list.
        let code_block = context.compile_with_new_declarative(&body, strict)?;
        // Indirect calls don't need extensions, because a non-strict indirect call modifies only
        // the global object.
        // Strict direct calls also don't need extensions, since all strict eval calls push a new
        // function environment before evaluating.
        if direct && !strict {
            context
                .realm
                .environments
                .extend_outer_function_environment();
        }
        let result = context.execute(code_block);

        match action {
            EnvStackAction::Truncate(size) => {
                context.realm.environments.truncate(size);
            }
            EnvStackAction::Restore(envs) => {
                // Pop all environments created during the eval execution and restore the original stack.
                context.realm.environments.truncate(1);
                context.realm.environments.extend(envs);
            }
        }

        result
    }
}
