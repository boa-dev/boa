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
    Context, JsResult, JsString, JsValue,
};
use boa_ast::operations::{
    contains, contains_arguments, top_level_var_declared_names, ContainsSymbol,
};
use boa_gc::Gc;
use boa_parser::Parser;
use boa_profiler::Profiler;

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
        bitflags::bitflags! {
            /// Flags used to throw early errors on invalid `eval` calls.
            #[derive(Default)]
            struct Flags: u8 {
                const IN_FUNCTION = 0b0001;
                const IN_METHOD = 0b0010;
                const IN_DERIVED_CONSTRUCTOR = 0b0100;
                const IN_CLASS_FIELD_INITIALIZER = 0b1000;
            }
        }

        /// Possible actions that can be executed after exiting this function to restore the environment to its
        /// original state.
        #[derive(Debug)]
        enum EnvStackAction {
            Truncate(usize),
            Restore(Vec<Gc<DeclarativeEnvironment>>),
        }

        /// Restores the environment after calling `eval` or after throwing an error.
        fn restore_environment(context: &mut Context, action: EnvStackAction) {
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
        }

        // 1. Assert: If direct is false, then strictCaller is also false.
        debug_assert!(direct || !strict);

        // 2. If Type(x) is not String, return x.
        // TODO: rework parser to take an iterator of `u32` unicode codepoints
        let Some(x) = x.as_string().map(JsString::to_std_string_escaped) else {
            return Ok(x.clone());
        };

        // Because of implementation details the following code differs from the spec.

        // 5. Perform ? HostEnsureCanCompileStrings(evalRealm).
        let mut parser = Parser::new(x.as_bytes());
        if strict {
            parser.set_strict();
        }
        // 11. Perform the following substeps in an implementation-defined order, possibly interleaving parsing and error detection:
        //     a. Let script be ParseText(StringToCodePoints(x), Script).
        //     b. If script is a List of errors, throw a SyntaxError exception.
        //     c. If script Contains ScriptBody is false, return undefined.
        //     d. Let body be the ScriptBody of script.
        let body = parser.parse_eval(direct, context.interner_mut())?;

        // 6. Let inFunction be false.
        // 7. Let inMethod be false.
        // 8. Let inDerivedConstructor be false.
        // 9. Let inClassFieldInitializer be false.
        // a. Let thisEnvRec be GetThisEnvironment().
        let flags = match context
            .realm
            .environments
            .get_this_environment()
            .as_function_slots()
        {
            // 10. If direct is true, then
            //     b. If thisEnvRec is a Function Environment Record, then
            Some(function_env) if direct => {
                let function_env = function_env.borrow();
                // i. Let F be thisEnvRec.[[FunctionObject]].
                let function_object = function_env.function_object().borrow();

                // ii. Set inFunction to true.
                let mut flags = Flags::IN_FUNCTION;

                // iii. Set inMethod to thisEnvRec.HasSuperBinding().
                if function_env.has_super_binding() {
                    flags |= Flags::IN_METHOD;
                }

                // iv. If F.[[ConstructorKind]] is derived, set inDerivedConstructor to true.
                if function_object
                    .as_function()
                    .expect("must be function object")
                    .is_derived_constructor()
                {
                    flags |= Flags::IN_DERIVED_CONSTRUCTOR;
                }

                // TODO:
                // v. Let classFieldInitializerName be F.[[ClassFieldInitializerName]].
                // vi. If classFieldInitializerName is not empty, set inClassFieldInitializer to true.

                flags
            }
            _ => Flags::default(),
        };

        if !flags.contains(Flags::IN_FUNCTION) && contains(&body, ContainsSymbol::NewTarget) {
            return Err(JsNativeError::syntax()
                .with_message("invalid `new.target` expression inside eval")
                .into());
        }
        if !flags.contains(Flags::IN_METHOD) && contains(&body, ContainsSymbol::SuperProperty) {
            return Err(JsNativeError::syntax()
                .with_message("invalid `super` reference inside eval")
                .into());
        }
        if !flags.contains(Flags::IN_DERIVED_CONSTRUCTOR)
            && contains(&body, ContainsSymbol::SuperCall)
        {
            return Err(JsNativeError::syntax()
                .with_message("invalid `super` call inside eval")
                .into());
        }
        if flags.contains(Flags::IN_CLASS_FIELD_INITIALIZER) && contains_arguments(&body) {
            return Err(JsNativeError::syntax()
                .with_message("invalid `arguments` reference inside eval")
                .into());
        }

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
            if let Some(name) = context
                .realm
                .environments
                .has_lex_binding_until_function_environment(&top_level_var_declared_names(&body))
            {
                restore_environment(context, action);
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

        restore_environment(context, action);

        result
    }
}
