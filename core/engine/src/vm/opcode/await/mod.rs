use boa_gc::{Gc, GcRefCell};

use crate::{
    builtins::{
        async_generator::AsyncGenerator, generator::GeneratorContext, promise::PromiseCapability,
        Promise,
    },
    native_function::NativeFunction,
    object::FunctionObjectBuilder,
    vm::{opcode::Operation, CompletionType, GeneratorResumeKind},
    Context, JsArgs, JsResult, JsValue,
};

/// `Await` implements the Opcode Operation for `Opcode::Await`
///
/// Operation:
///  - Stops the current Async function and schedules it to resume later.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Await;

impl Operation for Await {
    const NAME: &'static str = "Await";
    const INSTRUCTION: &'static str = "INST - Await";
    const COST: u8 = 5;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();

        // 2. Let promise be ? PromiseResolve(%Promise%, value).
        let promise = Promise::promise_resolve(
            &context.intrinsics().constructors().promise().constructor(),
            value,
            context,
        )?;

        let return_value = context
            .vm
            .frame()
            .promise_capability(&context.vm.stack)
            .as_ref()
            .map(PromiseCapability::promise)
            .cloned()
            .map(JsValue::from)
            .unwrap_or_default();

        let gen = GeneratorContext::from_current(context);

        let captures = Gc::new(GcRefCell::new(Some(gen)));

        // 3. Let fulfilledClosure be a new Abstract Closure with parameters (value) that captures asyncContext and performs the following steps when called:
        // 4. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 1, "", « »).
        let on_fulfilled = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_copy_closure_with_captures(
                |_this, args, captures, context| {
                    // a. Let prevContext be the running execution context.
                    // b. Suspend prevContext.
                    // c. Push asyncContext onto the execution context stack; asyncContext is now the running execution context.
                    // d. Resume the suspended evaluation of asyncContext using NormalCompletion(value) as the result of the operation that suspended it.
                    let mut gen = captures.borrow_mut().take().expect("should only run once");

                    // NOTE: We need to get the object before resuming, since it could clear the stack.
                    let async_generator = gen.async_generator_object();

                    gen.resume(
                        Some(args.get_or_undefined(0).clone()),
                        GeneratorResumeKind::Normal,
                        context,
                    );

                    if let Some(async_generator) = async_generator {
                        async_generator
                            .downcast_mut::<AsyncGenerator>()
                            .expect("must be async generator")
                            .context = Some(gen);
                    }

                    // e. Assert: When we reach this step, asyncContext has already been removed from the execution context stack and prevContext is the currently running execution context.
                    // f. Return undefined.
                    Ok(JsValue::undefined())
                },
                captures.clone(),
            ),
        )
        .name("")
        .length(1)
        .build();

        // 5. Let rejectedClosure be a new Abstract Closure with parameters (reason) that captures asyncContext and performs the following steps when called:
        // 6. Let onRejected be CreateBuiltinFunction(rejectedClosure, 1, "", « »).
        let on_rejected = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_copy_closure_with_captures(
                |_this, args, captures, context| {
                    // a. Let prevContext be the running execution context.
                    // b. Suspend prevContext.
                    // c. Push asyncContext onto the execution context stack; asyncContext is now the running execution context.
                    // d. Resume the suspended evaluation of asyncContext using ThrowCompletion(reason) as the result of the operation that suspended it.
                    // e. Assert: When we reach this step, asyncContext has already been removed from the execution context stack and prevContext is the currently running execution context.
                    // f. Return undefined.

                    let mut gen = captures.borrow_mut().take().expect("should only run once");

                    // NOTE: We need to get the object before resuming, since it could clear the stack.
                    let async_generator = gen.async_generator_object();

                    gen.resume(
                        Some(args.get_or_undefined(0).clone()),
                        GeneratorResumeKind::Throw,
                        context,
                    );

                    if let Some(async_generator) = async_generator {
                        async_generator
                            .downcast_mut::<AsyncGenerator>()
                            .expect("must be async generator")
                            .context = Some(gen);
                    }

                    Ok(JsValue::undefined())
                },
                captures,
            ),
        )
        .name("")
        .length(1)
        .build();

        // 7. Perform PerformPromiseThen(promise, onFulfilled, onRejected).
        Promise::perform_promise_then(
            &promise,
            Some(on_fulfilled),
            Some(on_rejected),
            None,
            context,
        );

        context.vm.set_return_value(return_value);
        Ok(CompletionType::Yield)
    }
}

/// `CreatePromiseCapability` implements the Opcode Operation for `Opcode::CreatePromiseCapability`
///
/// Operation:
///  - Create a promise capacity for an async function, if not already set.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreatePromiseCapability;

impl Operation for CreatePromiseCapability {
    const NAME: &'static str = "CreatePromiseCapability";
    const INSTRUCTION: &'static str = "INST - CreatePromiseCapability";
    const COST: u8 = 8;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        if context
            .vm
            .frame()
            .promise_capability(&context.vm.stack)
            .is_some()
        {
            return Ok(CompletionType::Normal);
        }

        let promise_capability = crate::builtins::promise::PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail per spec");

        context
            .vm
            .frames
            .last()
            .expect("there should be a frame")
            .set_promise_capability(&mut context.vm.stack, Some(&promise_capability));
        Ok(CompletionType::Normal)
    }
}

/// `CompletePromiseCapability` implements the Opcode Operation for `Opcode::CompletePromiseCapability`
///
/// Operation:
///  - Resolves or rejects the promise capability, depending if the pending exception is set.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CompletePromiseCapability;

impl Operation for CompletePromiseCapability {
    const NAME: &'static str = "CompletePromiseCapability";
    const INSTRUCTION: &'static str = "INST - CompletePromiseCapability";
    const COST: u8 = 8;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        // If the current executing function is an async function we have to resolve/reject it's promise at the end.
        // The relevant spec section is 3. in [AsyncBlockStart](https://tc39.es/ecma262/#sec-asyncblockstart).
        let Some(promise_capability) = context.vm.frame().promise_capability(&context.vm.stack)
        else {
            return if context.vm.pending_exception.is_some() {
                Ok(CompletionType::Throw)
            } else {
                Ok(CompletionType::Normal)
            };
        };

        if let Some(error) = context.vm.pending_exception.take() {
            promise_capability
                .reject()
                .call(&JsValue::undefined(), &[error.to_opaque(context)], context)
                .expect("cannot fail per spec");
        } else {
            let return_value = context.vm.get_return_value();
            promise_capability
                .resolve()
                .call(&JsValue::undefined(), &[return_value], context)
                .expect("cannot fail per spec");
        };

        context
            .vm
            .set_return_value(promise_capability.promise().clone().into());

        Ok(CompletionType::Normal)
    }
}
