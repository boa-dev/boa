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
    object::FunctionBuilder,
    property::Attribute,
    Context, JsValue,
};
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
    fn eval(_: &JsValue, args: &[JsValue], context: &mut Context) -> Result<JsValue, JsValue> {
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
        strict: bool,
        context: &mut Context,
    ) -> Result<JsValue, JsValue> {
        // 1. Assert: If direct is false, then strictCaller is also false.
        if !direct {
            debug_assert!(!strict);
        }

        // 2. If Type(x) is not String, return x.
        let x = if let Some(x) = x.as_string() {
            x.clone()
        } else {
            return Ok(x.clone());
        };

        // Because of implementation details the following code differs from the spec.

        // Parse the script body (11.a - 11.d)
        // TODO: Implement errors for 11.e - 11.h
        let parse_result = if strict {
            context.parse_strict(x.as_bytes())
        } else {
            context.parse(x.as_bytes())
        };
        let body = match parse_result.map_err(|e| e.to_string()) {
            Ok(body) => body,
            Err(e) => return context.throw_syntax_error(e),
        };

        // 12 - 13 are implicit in the call of `Context::compile_with_new_declarative`.

        // Because our environment model does not map directly to the spec this section looks very different.
        // 14 - 33 are in the following section, together with EvalDeclarationInstantiation.
        if direct {
            // If the call to eval is direct, the code is executed in the current environment.

            // Poison the current environment, because it may contain new declarations after/during eval.
            context.realm.environments.poison_current();

            // Set the compile time environment to the current running environment and save the number of current environments.
            context.realm.compile_env = context.realm.environments.current_compile_environment();
            let environments_len = context.realm.environments.len();

            // Error if any var declaration in the eval code already exists as a let/const declaration in the current running environment.
            let mut vars = FxHashSet::default();
            body.var_declared_names_new(&mut vars);
            if let Some(name) = context
                .realm
                .environments
                .has_lex_binding_until_function_environment(&vars)
            {
                let name = context.interner().resolve_expect(name);
                let msg = format!("variable declaration {name} in eval function already exists as lexically declaration");
                return context.throw_syntax_error(msg);
            }

            // Compile and execute the eval statement list.
            let code_block = context.compile_with_new_declarative(&body, strict)?;
            context
                .realm
                .environments
                .extend_outer_function_environment();
            let result = context.execute(code_block);

            // Pop any added runtime environments that where not removed during the eval execution.
            context.realm.environments.truncate(environments_len);

            result
        } else {
            // If the call to eval is indirect, the code is executed in the global environment.

            // Poison all environments, because the global environment may contain new declarations after/during eval.
            context.realm.environments.poison_all();

            // Pop all environments before the eval execution.
            let environments = context.realm.environments.pop_to_global();
            let environments_len = context.realm.environments.len();
            context.realm.compile_env = context.realm.environments.current_compile_environment();

            // Compile and execute the eval statement list.
            let code_block = context.compile_with_new_declarative(&body, false)?;
            let result = context.execute(code_block);

            // Restore all environments to the state from before the eval execution.
            context.realm.environments.truncate(environments_len);
            context.realm.environments.extend(environments);

            result
        }
    }
}
