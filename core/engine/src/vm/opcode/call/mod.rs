use std::{cell::RefCell, mem::MaybeUninit};

use boa_string::JsString;
use dynify::Dynify;

use super::VaryingOperand;
use crate::{
    Context, JsError, JsObject, JsResult, JsValue, NativeFunction,
    builtins::{Promise, promise::PromiseCapability},
    error::JsNativeError,
    job::NativeAsyncJob,
    module::{ModuleKind, Referrer},
    object::FunctionObjectBuilder,
    vm::opcode::Operation,
};

/// `CallEval` implements the Opcode Operation for `Opcode::CallEval`
///
/// Operation:
///  - Call a function named "eval".
#[derive(Debug, Clone, Copy)]
pub(crate) struct CallEval;

impl CallEval {
    #[inline(always)]
    pub(super) fn operation(
        (argument_count, scope_index): (VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let func = context
            .vm
            .stack
            .calling_convention_get_function(argument_count.into());

        let Some(object) = func.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("not a callable function")
                .into());
        };

        // Taken from `13.3.6.1 Runtime Semantics: Evaluation`
        //            `CallExpression : CoverCallExpressionAndAsyncArrowHead`
        //
        // <https://tc39.es/ecma262/#sec-function-calls-runtime-semantics-evaluation>
        //
        // 6. If ref is a Reference Record, IsPropertyReference(ref) is false, and ref.[[ReferencedName]] is "eval", then
        //     a. If SameValue(func, %eval%) is true, then
        let eval = context.intrinsics().objects().eval();
        if JsObject::equals(&object, &eval) {
            let arguments = context
                .vm
                .stack
                .calling_convention_pop_arguments(argument_count.into());
            let _func = context.vm.stack.pop();
            let _this = context.vm.stack.pop();
            if let Some(x) = arguments.first() {
                // i. Let argList be ? ArgumentListEvaluation of arguments.
                // ii. If argList has no elements, return undefined.
                // iii. Let evalArg be the first element of argList.
                // iv. If the source text matched by this CallExpression is strict mode code,
                //     let strictCaller be true. Otherwise let strictCaller be false.
                // v. Return ? PerformEval(evalArg, strictCaller, true).
                let strict = context.vm.frame().code_block.strict();
                let scope = context
                    .vm
                    .frame()
                    .code_block()
                    .constant_scope(scope_index.into());
                let result = crate::builtins::eval::Eval::perform_eval(
                    x,
                    true,
                    Some(scope),
                    strict,
                    context,
                )?;
                context.vm.stack.push(result);
            } else {
                // NOTE: This is a deviation from the spec, to optimize the case when we dont pass anything to `eval`.
                context.vm.stack.push(JsValue::undefined());
            }

            return Ok(());
        }

        object.__call__(argument_count.into()).resolve(context)?;
        Ok(())
    }
}

impl Operation for CallEval {
    const NAME: &'static str = "CallEval";
    const INSTRUCTION: &'static str = "INST - CallEval";
    const COST: u8 = 5;
}

/// `CallEvalSpread` implements the Opcode Operation for `Opcode::CallEvalSpread`
///
/// Operation:
///  - Call a function named "eval" where the arguments contain spreads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CallEvalSpread;

impl CallEvalSpread {
    #[inline(always)]
    pub(super) fn operation(index: VaryingOperand, context: &mut Context) -> JsResult<()> {
        // Get the arguments that are stored as an array object on the stack.
        let arguments_array = context.vm.stack.pop();
        let arguments_array_object = arguments_array
            .as_object()
            .expect("arguments array in call spread function must be an object");
        let arguments = arguments_array_object
            .borrow()
            .properties()
            .to_dense_indexed_properties()
            .expect("arguments array in call spread function must be dense");

        let func = context.vm.stack.calling_convention_get_function(0);

        let Some(object) = func.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("not a callable function")
                .into());
        };
        // Taken from `13.3.6.1 Runtime Semantics: Evaluation`
        //            `CallExpression : CoverCallExpressionAndAsyncArrowHead`
        //
        // <https://tc39.es/ecma262/#sec-function-calls-runtime-semantics-evaluation>
        //
        // 6. If ref is a Reference Record, IsPropertyReference(ref) is false, and ref.[[ReferencedName]] is "eval", then
        //     a. If SameValue(func, %eval%) is true, then
        let eval = context.intrinsics().objects().eval();
        if JsObject::equals(&object, &eval) {
            let _func = context.vm.stack.pop();
            let _this = context.vm.stack.pop();
            if let Some(x) = arguments.first() {
                // i. Let argList be ? ArgumentListEvaluation of arguments.
                // ii. If argList has no elements, return undefined.
                // iii. Let evalArg be the first element of argList.
                // iv. If the source text matched by this CallExpression is strict mode code,
                //     let strictCaller be true. Otherwise let strictCaller be false.
                // v. Return ? PerformEval(evalArg, strictCaller, true).
                let strict = context.vm.frame().code_block.strict();
                let scope = context.vm.frame().code_block().constant_scope(index.into());
                let result = crate::builtins::eval::Eval::perform_eval(
                    x,
                    true,
                    Some(scope),
                    strict,
                    context,
                )?;
                context.vm.stack.push(result);
            } else {
                // NOTE: This is a deviation from the spec, to optimize the case when we dont pass anything to `eval`.
                context.vm.stack.push(JsValue::undefined());
            }

            return Ok(());
        }

        let argument_count = arguments.len();
        context
            .vm
            .stack
            .calling_convention_push_arguments(&arguments);

        object.__call__(argument_count).resolve(context)?;
        Ok(())
    }
}

impl Operation for CallEvalSpread {
    const NAME: &'static str = "CallEvalSpread";
    const INSTRUCTION: &'static str = "INST - CallEvalSpread";
    const COST: u8 = 5;
}

/// `Call` implements the Opcode Operation for `Opcode::Call`
///
/// Operation:
///  - Call a function
#[derive(Debug, Clone, Copy)]
pub(crate) struct Call;

impl Call {
    #[inline(always)]
    pub(super) fn operation(argument_count: VaryingOperand, context: &mut Context) -> JsResult<()> {
        let func = context
            .vm
            .stack
            .calling_convention_get_function(argument_count.into());

        let Some(object) = func.as_object() else {
            return Err(Self::handle_not_callable());
        };

        object.__call__(argument_count.into()).resolve(context)?;

        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn handle_not_callable() -> JsError {
        JsNativeError::typ()
            .with_message("not a callable function")
            .into()
    }
}

impl Operation for Call {
    const NAME: &'static str = "Call";
    const INSTRUCTION: &'static str = "INST - Call";
    const COST: u8 = 3;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CallSpread;

impl CallSpread {
    #[inline(always)]
    pub(super) fn operation((): (), context: &mut Context) -> JsResult<()> {
        // Get the arguments that are stored as an array object on the stack.
        let arguments_array = context.vm.stack.pop();
        let arguments_array_object = arguments_array
            .as_object()
            .expect("arguments array in call spread function must be an object");
        let arguments = arguments_array_object
            .borrow()
            .properties()
            .to_dense_indexed_properties()
            .expect("arguments array in call spread function must be dense");

        let argument_count = arguments.len();
        context
            .vm
            .stack
            .calling_convention_push_arguments(&arguments);

        let func = context
            .vm
            .stack
            .calling_convention_get_function(argument_count);

        let Some(object) = func.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("not a callable function")
                .into());
        };

        object.__call__(argument_count).resolve(context)?;
        Ok(())
    }
}

impl Operation for CallSpread {
    const NAME: &'static str = "CallSpread";
    const INSTRUCTION: &'static str = "INST - CallSpread";
    const COST: u8 = 3;
}

/// Parses the import attributes from the options object.
fn parse_import_attributes(
    specifier: JsString,
    options: JsValue,
    context: &mut Context,
) -> JsResult<crate::module::ModuleRequest> {
    if options.is_undefined() {
        return Ok(crate::module::ModuleRequest::from_specifier(specifier));
    }

    if let Some(options_obj) = options.as_object() {
        let with_key = crate::js_string!("with");
        let with_val = options_obj.get(with_key, context)?;

        let (attributes_val, is_assert) = if !with_val.is_undefined() {
            (with_val, false)
        } else {
            let assert_key = crate::js_string!("assert");
            let assert_val = options_obj.get(assert_key, context)?;
            (assert_val, true)
        };

        if !attributes_val.is_undefined() {
            if let Some(attributes_obj) = attributes_val.as_object() {
                let keys = attributes_obj.enumerable_own_property_names(
                    crate::property::PropertyNameKind::Key,
                    context,
                )?;

                let mut attributes = Vec::with_capacity(keys.len());
                for key in keys {
                    if !key.is_string() {
                        continue;
                    }
                    let key_str = key.as_string().expect("key must be string").clone();
                    let value = attributes_obj.get(key_str.clone(), context)?;

                    if !value.is_string() {
                        return Err(JsNativeError::typ()
                            .with_message("import attribute value must be a string")
                            .into());
                    }

                    let value_str = value.as_string().expect("value must be string").clone();
                    attributes.push((key_str, value_str));
                }
                Ok(crate::module::ModuleRequest::new(
                    specifier,
                    attributes.into_boxed_slice(),
                ))
            } else {
                let msg = if is_assert {
                    "the 'assert' option must be an object"
                } else {
                    "the 'with' option must be an object"
                };
                Err(JsNativeError::typ().with_message(msg).into())
            }
        } else {
            Ok(crate::module::ModuleRequest::from_specifier(specifier))
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("import options must be an object or undefined")
            .into())
    }
}

/// Loads the module of a dynamic import. This combines the operations:
/// - [`HostLoadImportedModule(referrer, specifierString, empty, promiseCapability).`][load]
/// - [`FinishLoadingImportedModule ( referrer, specifier, payload, result )`][finish]
/// - [`ContinueDynamicImport ( promiseCapability, moduleCompletion )`][continue]
///
/// [load]: https://tc39.es/ecma262/#sec-HostLoadImportedModule
/// [finish]: https://tc39.es/ecma262/#sec-FinishLoadingImportedModule
/// [continue]: https://tc39.es/ecma262/#sec-ContinueDynamicImport
async fn load_dyn_import(
    referrer: Referrer,
    specifier: JsString,
    options: JsValue,
    cap: PromiseCapability,
    context: &RefCell<&mut Context>,
) -> JsResult<()> {
    let request = {
        let mut context = context.borrow_mut();
        parse_import_attributes(specifier, options, &mut context)
    };

    let request = match request {
        Ok(req) => req,
        Err(err) => {
            let err = err.into_opaque(&mut context.borrow_mut())?;
            cap.reject()
                .call(&JsValue::undefined(), &[err], &mut context.borrow_mut())
                .expect("default `reject` function cannot throw");
            return Ok(());
        }
    };

    let loader = context.borrow().module_loader();
    let fut = loader.load_imported_module(referrer.clone(), request.clone(), context);
    let mut stack = [MaybeUninit::<u8>::uninit(); 16];
    let mut heap = Vec::<MaybeUninit<u8>>::new();
    let completion = fut.init2(&mut stack, &mut heap).await;

    let module = match completion {
        Err(err) => {
            let err = err.into_opaque(&mut context.borrow_mut())?;
            cap.reject()
                .call(&JsValue::undefined(), &[err], &mut context.borrow_mut())
                .expect("default `reject` function cannot throw");
            return Ok(());
        }
        Ok(m) => m,
    };

    match referrer {
        Referrer::Module(mod_ref) => {
            let ModuleKind::SourceText(src) = mod_ref.kind() else {
                panic!("referrer cannot be a synthetic module");
            };

            let mut loaded_modules = src.loaded_modules().borrow_mut();
            let entry = loaded_modules
                .entry(request)
                .or_insert_with(|| module.clone());

            debug_assert_eq!(&module, entry);
        }
        Referrer::Realm(realm) => {
            let mut loaded_modules = realm.loaded_modules().borrow_mut();
            let entry = loaded_modules
                .entry(request.specifier().clone())
                .or_insert_with(|| module.clone());
            debug_assert_eq!(&module, entry);
        }
        Referrer::Script(script) => {
            let mut loaded_modules = script.loaded_modules().borrow_mut();
            let entry = loaded_modules
                .entry(request.specifier().clone())
                .or_insert_with(|| module.clone());
            debug_assert_eq!(&module, entry);
        }
    }

    // 2. Let module be moduleCompletion.[[Value]].
    // 3. Let loadPromise be module.LoadRequestedModules().
    let load = module.load(&mut context.borrow_mut());

    let on_rejected = FunctionObjectBuilder::new(
        context.borrow().realm(),
        NativeFunction::from_copy_closure_with_captures(
            |_, args, cap, context| {
                cap.reject()
                    .call(&JsValue::undefined(), args, context)
                    .expect("default `reject` function cannot throw");
                Ok(JsValue::undefined())
            },
            cap.clone(),
        ),
    )
    .build();

    let link_evaluate = FunctionObjectBuilder::new(
        context.borrow().realm(),
        NativeFunction::from_copy_closure_with_captures(
            |_, _, (module, cap, on_rejected), context| {
                if let Err(e) = module.link(context) {
                    let e = e.into_opaque(context)?;
                    cap.reject()
                        .call(&JsValue::undefined(), &[e], context)
                        .expect("default `reject` function cannot throw");
                    return Ok(JsValue::undefined());
                }

                let evaluate = module.evaluate(context)?;

                let fulfill = FunctionObjectBuilder::new(
                    context.realm(),
                    NativeFunction::from_copy_closure_with_captures(
                        |_, _, (module, cap), context| {
                            let namespace = module.namespace(context);
                            cap.resolve()
                                .call(&JsValue::undefined(), &[namespace.into()], context)
                                .expect("default `resolve` function cannot throw");
                            Ok(JsValue::undefined())
                        },
                        (module.clone(), cap.clone()),
                    ),
                )
                .build();

                Promise::perform_promise_then(
                    &evaluate,
                    Some(fulfill),
                    Some(on_rejected.clone()),
                    None,
                    context,
                );

                Ok(JsValue::undefined())
            },
            (module.clone(), cap.clone(), on_rejected.clone()),
        ),
    )
    .build();

    Promise::perform_promise_then(
        &load,
        Some(link_evaluate),
        Some(on_rejected),
        None,
        &mut context.borrow_mut(),
    );

    Ok(())
}

/// `ImportCall` implements the Opcode Operation for `Opcode::ImportCall`
///
/// Operation:
///  - Dynamically imports a module
#[derive(Debug, Clone, Copy)]
pub(crate) struct ImportCall;

impl ImportCall {
    #[inline(always)]
    pub(super) fn operation(
        (specifier_op, options_op): (VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        // Import Calls
        // Runtime Semantics: Evaluation
        // https://tc39.es/ecma262/#sec-import-call-runtime-semantics-evaluation

        // 1. Let referrer be GetActiveScriptOrModule().
        // 2. If referrer is null, set referrer to the current Realm Record.
        let referrer = context
            .get_active_script_or_module()
            .map_or_else(|| Referrer::Realm(context.realm().clone()), Into::into);

        // 3. Let argRef be ? Evaluation of AssignmentExpression.
        // 4. Let specifier be ? GetValue(argRef).
        let arg = context.vm.get_register(specifier_op.into()).clone();

        // Get options if provided
        let options = context.vm.get_register(options_op.into()).clone();

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
                let err = err.into_opaque(context)?;
                cap.reject().call(&JsValue::undefined(), &[err], context)?;
            }
            // 8. Perform HostLoadImportedModule(referrer, specifierString, empty, promiseCapability).
            Ok(specifier) => {
                let job = NativeAsyncJob::with_realm(
                    async move |context| {
                        load_dyn_import(referrer, specifier, options, cap, context).await?;
                        Ok(JsValue::undefined())
                    },
                    context.realm().clone(),
                );
                context.enqueue_job(job.into());
            }
        }

        // 9. Return promiseCapability.[[Promise]].
        context.vm.set_register(specifier_op.into(), promise.into());

        Ok(())
    }
}

impl Operation for ImportCall {
    const NAME: &'static str = "ImportCall";
    const INSTRUCTION: &'static str = "INST - ImportCall";
    const COST: u8 = 15;
}
