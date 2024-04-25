//! Boa's implementation of ECMAScript's global `eval` function.
//!
//! The `eval()` function evaluates ECMAScript code represented as a string.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-eval-x
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval

use crate::{
    builtins::{function::OrdinaryFunction, BuiltInObject},
    bytecompiler::{eval_declaration_instantiation_context, ByteCompiler},
    context::intrinsics::Intrinsics,
    environments::Environment,
    error::JsNativeError,
    js_string,
    object::JsObject,
    realm::Realm,
    string::common::StaticJsStrings,
    vm::{CallFrame, CallFrameFlags, Opcode},
    Context, JsArgs, JsResult, JsString, JsValue,
};
use boa_ast::operations::{contains, contains_arguments, ContainsSymbol};
use boa_gc::Gc;
use boa_parser::{Parser, Source};
use boa_profiler::Profiler;

use super::{BuiltInBuilder, IntrinsicObject};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Eval;

impl IntrinsicObject for Eval {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, Self::eval)
            .name(Self::NAME)
            .length(1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().eval().into()
    }
}

impl BuiltInObject for Eval {
    const NAME: JsString = StaticJsStrings::EVAL;
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
            Restore(Vec<Environment>),
        }

        // 1. Assert: If direct is false, then strictCaller is also false.
        debug_assert!(direct || !strict);

        // 2. If Type(x) is not String, return x.
        // TODO: rework parser to take an iterator of `u32` unicode codepoints
        let Some(x) = x.as_string() else {
            return Ok(x.clone());
        };

        // Because of implementation details the following code differs from the spec.

        // 3. Let evalRealm be the current Realm Record.
        // 4. NOTE: In the case of a direct eval, evalRealm is the realm of both the caller of eval
        // and of the eval function itself.
        let eval_realm = context.realm().clone();

        // 5. Perform ? HostEnsureCanCompileStrings(evalRealm, « », x, direct).
        context
            .host_hooks()
            .ensure_can_compile_strings(eval_realm, &[], x, direct, context)?;

        // 11. Perform the following substeps in an implementation-defined order, possibly interleaving parsing and error detection:
        //     a. Let script be ParseText(StringToCodePoints(x), Script).
        //     b. If script is a List of errors, throw a SyntaxError exception.
        //     c. If script Contains ScriptBody is false, return undefined.
        //     d. Let body be the ScriptBody of script.
        let mut parser = Parser::new(Source::from_utf16(x));
        parser.set_identifier(context.next_parser_identifier());
        if strict {
            parser.set_strict();
        }
        let body = parser.parse_eval(direct, context.interner_mut())?;

        // 6. Let inFunction be false.
        // 7. Let inMethod be false.
        // 8. Let inDerivedConstructor be false.
        // 9. Let inClassFieldInitializer be false.
        // a. Let thisEnvRec be GetThisEnvironment().
        let flags = match context.vm.environments.get_this_environment().as_function() {
            // 10. If direct is true, then
            //     b. If thisEnvRec is a Function Environment Record, then
            Some(function_env) if direct => {
                // i. Let F be thisEnvRec.[[FunctionObject]].
                let function_object = function_env.slots().function_object().borrow();

                // ii. Set inFunction to true.
                let mut flags = Flags::IN_FUNCTION;

                // iii. Set inMethod to thisEnvRec.HasSuperBinding().
                if function_env.has_super_binding() {
                    flags |= Flags::IN_METHOD;
                }

                let function_object = function_object
                    .downcast_ref::<OrdinaryFunction>()
                    .expect("must be function object");

                // iv. If F.[[ConstructorKind]] is derived, set inDerivedConstructor to true.
                if function_object.is_derived_constructor() {
                    flags |= Flags::IN_DERIVED_CONSTRUCTOR;
                }

                // v. Let classFieldInitializerName be F.[[ClassFieldInitializerName]].
                // vi. If classFieldInitializerName is not empty, set inClassFieldInitializer to true.
                if function_object.in_class_field_initializer() {
                    flags |= Flags::IN_CLASS_FIELD_INITIALIZER;
                }

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

            // Poison the last parent function environment, because it may contain new declarations after/during eval.
            if !strict {
                context.vm.environments.poison_until_last_function();
            }

            // Set the compile time environment to the current running environment and save the number of current environments.
            let environments_len = context.vm.environments.len();

            // Pop any added runtime environments that were not removed during the eval execution.
            EnvStackAction::Truncate(environments_len)
        } else {
            // If the call to eval is indirect, the code is executed in the global environment.

            // Pop all environments before the eval execution.
            let environments = context.vm.environments.pop_to_global();

            // Restore all environments to the state from before the eval execution.
            EnvStackAction::Restore(environments)
        };

        let context = &mut context.guard(move |ctx| match action {
            EnvStackAction::Truncate(len) => ctx.vm.environments.truncate(len),
            EnvStackAction::Restore(envs) => {
                ctx.vm.environments.truncate(1);
                ctx.vm.environments.extend(envs);
            }
        });

        let var_environment = context.vm.environments.outer_function_environment();
        let mut var_env = var_environment.compile_env();

        let mut compiler = ByteCompiler::new(
            js_string!("<main>"),
            body.strict(),
            false,
            var_env.clone(),
            context.vm.environments.current_compile_environment(),
            context,
        );

        let env_index = compiler.push_compile_environment(strict);
        compiler.emit_with_varying_operand(Opcode::PushDeclarativeEnvironment, env_index);
        let lex_env = compiler.lexical_environment.clone();
        if strict {
            var_env = lex_env.clone();
            compiler.variable_environment = lex_env.clone();
        }

        let mut annex_b_function_names = Vec::new();

        eval_declaration_instantiation_context(
            &mut annex_b_function_names,
            &body,
            strict,
            &var_env,
            &lex_env,
            compiler.context,
        )?;

        #[cfg(feature = "annex-b")]
        {
            compiler.annex_b_function_names = annex_b_function_names;
        }

        compiler.eval_declaration_instantiation(&body, strict, &var_env, &lex_env);
        compiler.compile_statement_list(body.statements(), true, false);

        let code_block = Gc::new(compiler.finish());

        // Strict calls don't need extensions, since all strict eval calls push a new
        // function environment before evaluating.
        if !strict {
            var_environment.extend_from_compile();
        }

        let env_fp = context.vm.environments.len() as u32;
        let environments = context.vm.environments.clone();
        let realm = context.realm().clone();
        context.vm.push_frame_with_stack(
            CallFrame::new(code_block, None, environments, realm)
                .with_env_fp(env_fp)
                .with_flags(CallFrameFlags::EXIT_EARLY),
            JsValue::undefined(),
            JsValue::null(),
        );

        context.realm().resize_global_env();

        let record = context.run();
        context.vm.pop_frame();

        record.consume()
    }
}
