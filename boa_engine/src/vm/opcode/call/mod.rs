use crate::{
    builtins::{function::FunctionKind, promise::PromiseCapability, Promise},
    error::JsNativeError,
    module::{ModuleKind, Referrer},
    object::FunctionObjectBuilder,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue, NativeFunction,
};

/// `CallEval` implements the Opcode Operation for `Opcode::CallEval`
///
/// Operation:
///  - Call a function named "eval".
#[derive(Debug, Clone, Copy)]
pub(crate) struct CallEval;

impl Operation for CallEval {
    const NAME: &'static str = "CallEval";
    const INSTRUCTION: &'static str = "INST - CallEval";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        if context.vm.runtime_limits.recursion_limit() <= context.vm.frames.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message(format!(
                    "Maximum recursion limit {} exceeded",
                    context.vm.runtime_limits.recursion_limit()
                ))
                .into());
        }
        if context.vm.runtime_limits.stack_size_limit() <= context.vm.stack.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message("Maximum call stack size exceeded")
                .into());
        }
        let argument_count = context.vm.read::<u32>();
        let mut arguments = Vec::with_capacity(argument_count as usize);
        for _ in 0..argument_count {
            arguments.push(context.vm.pop());
        }
        arguments.reverse();

        let func = context.vm.pop();
        let this = context.vm.pop();

        let object = match func {
            JsValue::Object(ref object) if object.is_callable() => object.clone(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("not a callable function")
                    .into());
            }
        };

        // A native function with the name "eval" implies, that is this the built-in eval function.
        let eval = object
            .borrow()
            .as_function()
            .map(|f| matches!(f.kind(), FunctionKind::Native { .. }))
            .unwrap_or_default();

        let strict = context.vm.frame().code_block.strict;

        if eval {
            if let Some(x) = arguments.get(0) {
                let result = crate::builtins::eval::Eval::perform_eval(x, true, strict, context)?;
                context.vm.push(result);
            } else {
                context.vm.push(JsValue::Undefined);
            }
        } else {
            let result = object.__call__(&this, &arguments, context)?;
            context.vm.push(result);
        }
        Ok(CompletionType::Normal)
    }
}

/// `CallEvalSpread` implements the Opcode Operation for `Opcode::CallEvalSpread`
///
/// Operation:
///  - Call a function named "eval" where the arguments contain spreads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CallEvalSpread;

impl Operation for CallEvalSpread {
    const NAME: &'static str = "CallEvalSpread";
    const INSTRUCTION: &'static str = "INST - CallEvalSpread";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        if context.vm.runtime_limits.recursion_limit() <= context.vm.frames.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message(format!(
                    "Maximum recursion limit {} exceeded",
                    context.vm.runtime_limits.recursion_limit()
                ))
                .into());
        }
        if context.vm.runtime_limits.stack_size_limit() <= context.vm.stack.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message("Maximum call stack size exceeded")
                .into());
        }

        // Get the arguments that are stored as an array object on the stack.
        let arguments_array = context.vm.pop();
        let arguments_array_object = arguments_array
            .as_object()
            .expect("arguments array in call spread function must be an object");
        let arguments = arguments_array_object
            .borrow()
            .properties()
            .dense_indexed_properties()
            .expect("arguments array in call spread function must be dense")
            .clone();

        let func = context.vm.pop();
        let this = context.vm.pop();

        let object = match func {
            JsValue::Object(ref object) if object.is_callable() => object.clone(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("not a callable function")
                    .into());
            }
        };

        // A native function with the name "eval" implies, that is this the built-in eval function.
        let eval = object
            .borrow()
            .as_function()
            .map(|f| matches!(f.kind(), FunctionKind::Native { .. }))
            .unwrap_or_default();

        let strict = context.vm.frame().code_block.strict;

        if eval {
            if let Some(x) = arguments.get(0) {
                let result = crate::builtins::eval::Eval::perform_eval(x, true, strict, context)?;
                context.vm.push(result);
            } else {
                context.vm.push(JsValue::Undefined);
            }
        } else {
            let result = object.__call__(&this, &arguments, context)?;
            context.vm.push(result);
        }
        Ok(CompletionType::Normal)
    }
}

/// `Call` implements the Opcode Operation for `Opcode::Call`
///
/// Operation:
///  - Call a function
#[derive(Debug, Clone, Copy)]
pub(crate) struct Call;

impl Operation for Call {
    const NAME: &'static str = "Call";
    const INSTRUCTION: &'static str = "INST - Call";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        if context.vm.runtime_limits.recursion_limit() <= context.vm.frames.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message(format!(
                    "Maximum recursion limit {} exceeded",
                    context.vm.runtime_limits.recursion_limit()
                ))
                .into());
        }
        if context.vm.runtime_limits.stack_size_limit() <= context.vm.stack.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message("Maximum call stack size exceeded")
                .into());
        }
        let argument_count = context.vm.read::<u32>();
        let mut arguments = Vec::with_capacity(argument_count as usize);
        for _ in 0..argument_count {
            arguments.push(context.vm.pop());
        }
        arguments.reverse();

        let func = context.vm.pop();
        let this = context.vm.pop();

        let object = match func {
            JsValue::Object(ref object) if object.is_callable() => object.clone(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("not a callable function")
                    .into());
            }
        };

        let result = object.__call__(&this, &arguments, context)?;

        context.vm.push(result);
        Ok(CompletionType::Normal)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CallSpread;

impl Operation for CallSpread {
    const NAME: &'static str = "CallSpread";
    const INSTRUCTION: &'static str = "INST - CallSpread";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        if context.vm.runtime_limits.recursion_limit() <= context.vm.frames.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message(format!(
                    "Maximum recursion limit {} exceeded",
                    context.vm.runtime_limits.recursion_limit()
                ))
                .into());
        }
        if context.vm.runtime_limits.stack_size_limit() <= context.vm.stack.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message("Maximum call stack size exceeded")
                .into());
        }

        // Get the arguments that are stored as an array object on the stack.
        let arguments_array = context.vm.pop();
        let arguments_array_object = arguments_array
            .as_object()
            .expect("arguments array in call spread function must be an object");
        let arguments = arguments_array_object
            .borrow()
            .properties()
            .dense_indexed_properties()
            .expect("arguments array in call spread function must be dense")
            .clone();

        let func = context.vm.pop();
        let this = context.vm.pop();

        let object = match func {
            JsValue::Object(ref object) if object.is_callable() => object.clone(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("not a callable function")
                    .into())
            }
        };

        let result = object.__call__(&this, &arguments, context)?;

        context.vm.push(result);
        Ok(CompletionType::Normal)
    }
}

/// `ImportCall` implements the Opcode Operation for `Opcode::ImportCall`
///
/// Operation:
///  - Dynamically imports a module
#[derive(Debug, Clone, Copy)]
pub(crate) struct ImportCall;

impl Operation for ImportCall {
    const NAME: &'static str = "ImportCall";
    const INSTRUCTION: &'static str = "INST - ImportCall";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        // Import Calls
        // Runtime Semantics: Evaluation
        // https://tc39.es/ecma262/#sec-import-call-runtime-semantics-evaluation

        // 1. Let referrer be GetActiveScriptOrModule().
        // 2. If referrer is null, set referrer to the current Realm Record.
        let referrer = context
            .vm
            .active_runnable
            .clone()
            .map_or_else(|| Referrer::Realm(context.realm().clone()), Into::into);

        // 3. Let argRef be ? Evaluation of AssignmentExpression.
        // 4. Let specifier be ? GetValue(argRef).
        let arg = context.vm.pop();

        // 5. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let cap = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("operation cannot fail for the %Promise% intrinsic");
        let promise = cap.promise().clone();

        // 6. Let specifierString be Completion(ToString(specifier)).
        match arg.to_string(context) {
            // 7. IfAbruptRejectPromise(specifierString, promiseCapability).
            Err(err) => {
                let err = err.to_opaque(context);
                cap.reject().call(&JsValue::undefined(), &[err], context)?;
            }
            // 8. Perform HostLoadImportedModule(referrer, specifierString, empty, promiseCapability).
            Ok(specifier) => context.module_loader().load_imported_module(
                referrer.clone(),
                specifier.clone(),
                Box::new(move |completion, context| {
                    // `ContinueDynamicImport ( promiseCapability, moduleCompletion )`
                    // https://tc39.es/ecma262/#sec-ContinueDynamicImport

                    // `FinishLoadingImportedModule ( referrer, specifier, payload, result )`
                    // https://tc39.es/ecma262/#sec-FinishLoadingImportedModule
                    let module = match completion {
                        // 1. If result is a normal completion, then
                        Ok(m) => {
                            match referrer {
                                Referrer::Module(module) => {
                                    let ModuleKind::SourceText(src) = module.kind() else {
                                        panic!("referrer cannot be a synthetic module");
                                    };

                                    let sym = context.interner_mut().get_or_intern(&*specifier);

                                    let mut loaded_modules = src.loaded_modules().borrow_mut();

                                    //     a. If referrer.[[LoadedModules]] contains a Record whose [[Specifier]] is specifier, then
                                    //     b. Else,
                                    //         i. Append the Record { [[Specifier]]: specifier, [[Module]]: result.[[Value]] } to referrer.[[LoadedModules]].
                                    let entry =
                                        loaded_modules.entry(sym).or_insert_with(|| m.clone());

                                    //         i. Assert: That Record's [[Module]] is result.[[Value]].
                                    debug_assert_eq!(&m, entry);

                                    // Same steps apply to referrers below
                                }
                                Referrer::Realm(realm) => {
                                    let mut loaded_modules = realm.loaded_modules().borrow_mut();
                                    let entry = loaded_modules
                                        .entry(specifier)
                                        .or_insert_with(|| m.clone());
                                    debug_assert_eq!(&m, entry);
                                }
                                Referrer::Script(script) => {
                                    let mut loaded_modules = script.loaded_modules().borrow_mut();
                                    let entry = loaded_modules
                                        .entry(specifier)
                                        .or_insert_with(|| m.clone());
                                    debug_assert_eq!(&m, entry);
                                }
                            }

                            m
                        }
                        // 1. If moduleCompletion is an abrupt completion, then
                        Err(err) => {
                            // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « moduleCompletion.[[Value]] »).
                            let err = err.to_opaque(context);
                            cap.reject()
                                .call(&JsValue::undefined(), &[err], context)
                                .expect("default `reject` function cannot throw");

                            // b. Return unused.
                            return;
                        }
                    };

                    // 2. Let module be moduleCompletion.[[Value]].
                    // 3. Let loadPromise be module.LoadRequestedModules().
                    let load = module.load(context);

                    // 4. Let rejectedClosure be a new Abstract Closure with parameters (reason) that captures promiseCapability and performs the following steps when called:
                    // 5. Let onRejected be CreateBuiltinFunction(rejectedClosure, 1, "", « »).
                    let on_rejected = FunctionObjectBuilder::new(
                        context,
                        NativeFunction::from_copy_closure_with_captures(
                            |_, args, cap, context| {
                                //     a. Perform ! Call(promiseCapability.[[Reject]], undefined, « reason »).
                                cap.reject()
                                    .call(&JsValue::undefined(), args, context)
                                    .expect("default `reject` function cannot throw");

                                //     b. Return unused.
                                Ok(JsValue::undefined())
                            },
                            cap.clone(),
                        ),
                    )
                    .build();

                    // 6. Let linkAndEvaluateClosure be a new Abstract Closure with no parameters that captures module, promiseCapability, and onRejected and performs the following steps when called:
                    // 7. Let linkAndEvaluate be CreateBuiltinFunction(linkAndEvaluateClosure, 0, "", « »).
                    let link_evaluate = FunctionObjectBuilder::new(
                        context,
                        NativeFunction::from_copy_closure_with_captures(
                            |_, _, (module, cap, on_rejected), context| {
                                // a. Let link be Completion(module.Link()).
                                // b. If link is an abrupt completion, then
                                if let Err(e) = module.link(context) {
                                    // i. Perform ! Call(promiseCapability.[[Reject]], undefined, « link.[[Value]] »).
                                    let e = e.to_opaque(context);
                                    cap.reject()
                                        .call(&JsValue::undefined(), &[e], context)
                                        .expect("default `reject` function cannot throw");
                                    // ii. Return unused.
                                    return Ok(JsValue::undefined());
                                }

                                // c. Let evaluatePromise be module.Evaluate().
                                let evaluate = module.evaluate(context);

                                // d. Let fulfilledClosure be a new Abstract Closure with no parameters that captures module and promiseCapability and performs the following steps when called:
                                // e. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 0, "", « »).
                                let fulfill = FunctionObjectBuilder::new(
                                    context,
                                    NativeFunction::from_copy_closure_with_captures(
                                        |_, _, (module, cap), context| {
                                            // i. Let namespace be GetModuleNamespace(module).
                                            let namespace = module.namespace(context);

                                            // ii. Perform ! Call(promiseCapability.[[Resolve]], undefined, « namespace »).
                                            cap.resolve()
                                                .call(
                                                    &JsValue::undefined(),
                                                    &[namespace.into()],
                                                    context,
                                                )
                                                .expect("default `resolve` function cannot throw");

                                            // iii. Return unused.
                                            Ok(JsValue::undefined())
                                        },
                                        (module.clone(), cap.clone()),
                                    ),
                                )
                                .build();

                                // f. Perform PerformPromiseThen(evaluatePromise, onFulfilled, onRejected).
                                Promise::perform_promise_then(
                                    &evaluate,
                                    Some(fulfill),
                                    Some(on_rejected.clone()),
                                    None,
                                    context,
                                );

                                // g. Return unused.
                                Ok(JsValue::undefined())
                            },
                            (module.clone(), cap.clone(), on_rejected.clone()),
                        ),
                    )
                    .build();

                    // 8. Perform PerformPromiseThen(loadPromise, linkAndEvaluate, onRejected).
                    Promise::perform_promise_then(
                        &load,
                        Some(link_evaluate),
                        Some(on_rejected),
                        None,
                        context,
                    );

                    // 9. Return unused.
                }),
                context,
            ),
        };

        // 9. Return promiseCapability.[[Promise]].
        context.vm.push(promise);

        Ok(CompletionType::Normal)
    }
}
