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
    builtins::{function::Function, BuiltIn, JsArgs},
    object::{JsObject, ObjectData},
    property::Attribute,
    Context, JsValue,
};
use boa_profiler::Profiler;
use rustc_hash::FxHashSet;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Eval;

impl BuiltIn for Eval {
    const NAME: &'static str = "eval";

    const ATTRIBUTE: Attribute = Attribute::READONLY
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::PERMANENT);

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let object = JsObject::from_proto_and_data(
            context.intrinsics().constructors().function().prototype(),
            ObjectData::function(Function::Native {
                function: Self::eval,
                constructor: false,
            }),
        );

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

    /// `19.2.1.1 PerformEval ( x, callerRealm, strictCaller, direct )`
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
        let x = if let Some(x) = x.as_string() {
            x.clone()
        } else {
            return Ok(x.clone());
        };

        let parsing_result = context.parse(x.as_bytes()).map_err(|e| e.to_string());

        let statement_list = match parsing_result {
            Ok(statement_list) => statement_list,
            Err(e) => return context.throw_syntax_error(e),
        };

        if direct {
            context.realm.environments.poison_current();
            context.realm.compile_env = context.realm.environments.current_compile_environment();
            let environments_len = context.realm.environments.len();

            let mut vars = FxHashSet::default();
            statement_list.var_declared_names_new(&mut vars);
            if let Some(name) = context
                .realm
                .environments
                .has_lex_binding_until_function_environment(&vars)
            {
                return context.throw_syntax_error(format!("variable declaration {} in eval function already exists as lexically declaration", context.interner().resolve_expect(name)));
            }

            let code_block = context.compile_with_new_declarative(&statement_list, strict)?;
            context
                .realm
                .environments
                .extend_outer_function_environment();
            let result = context.execute(code_block);

            context.realm.environments.truncate(environments_len);

            result
        } else {
            context.realm.environments.poison_all();

            let environments = context.realm.environments.pop_to_global();
            let environments_len = context.realm.environments.len();

            context.realm.compile_env = context.realm.environments.current_compile_environment();

            let code_block = context.compile_with_new_declarative(&statement_list, false)?;
            let result = context.execute(code_block);

            context.realm.environments.truncate(environments_len);
            context.realm.environments.extend(environments);

            result
        }
    }
}
