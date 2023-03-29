use boa_gc::{Gc, GcRefCell};

use crate::{
    builtins::Promise,
    native_function::NativeFunction,
    object::FunctionObjectBuilder,
    vm::{
        call_frame::{EarlyReturnType, GeneratorResumeKind},
        opcode::Operation,
        CompletionType,
    },
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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.pop();

        // 2. Let promise be ? PromiseResolve(%Promise%, value).
        let promise = Promise::promise_resolve(
            &context.intrinsics().constructors().promise().constructor(),
            value,
            context,
        )?;

        // 3. Let fulfilledClosure be a new Abstract Closure with parameters (value) that captures asyncContext and performs the following steps when called:
        // 4. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 1, "", « »).
        let on_fulfilled = FunctionObjectBuilder::new(
            context,
            NativeFunction::from_copy_closure_with_captures(
                |_this, args, captures, context| {
                    let mut captures = captures.borrow_mut();
                    let (environment, stack, frame) = &mut *captures;
                    // a. Let prevContext be the running execution context.
                    // b. Suspend prevContext.
                    // c. Push asyncContext onto the execution context stack; asyncContext is now the running execution context.
                    // d. Resume the suspended evaluation of asyncContext using NormalCompletion(value) as the result of the operation that suspended it.
                    // e. Assert: When we reach this step, asyncContext has already been removed from the execution context stack and prevContext is the currently running execution context.
                    // f. Return undefined.

                    std::mem::swap(&mut context.realm.environments, environment);
                    std::mem::swap(&mut context.vm.stack, stack);
                    context.vm.push_frame(frame.clone());

                    context.vm.frame_mut().generator_resume_kind = GeneratorResumeKind::Normal;
                    context.vm.push(args.get_or_undefined(0).clone());
                    context.run();

                    *frame = context
                        .vm
                        .pop_frame()
                        .expect("generator call frame must exist");
                    std::mem::swap(&mut context.realm.environments, environment);
                    std::mem::swap(&mut context.vm.stack, stack);

                    Ok(JsValue::undefined())
                },
                Gc::new(GcRefCell::new((
                    context.realm.environments.clone(),
                    context.vm.stack.clone(),
                    context.vm.frame().clone(),
                ))),
            ),
        )
        .name("")
        .length(1)
        .build();

        // 5. Let rejectedClosure be a new Abstract Closure with parameters (reason) that captures asyncContext and performs the following steps when called:
        // 6. Let onRejected be CreateBuiltinFunction(rejectedClosure, 1, "", « »).
        let on_rejected = FunctionObjectBuilder::new(
            context,
            NativeFunction::from_copy_closure_with_captures(
                |_this, args, captures, context| {
                    let mut captures = captures.borrow_mut();
                    let (environment, stack, frame) = &mut *captures;
                    // a. Let prevContext be the running execution context.
                    // b. Suspend prevContext.
                    // c. Push asyncContext onto the execution context stack; asyncContext is now the running execution context.
                    // d. Resume the suspended evaluation of asyncContext using ThrowCompletion(reason) as the result of the operation that suspended it.
                    // e. Assert: When we reach this step, asyncContext has already been removed from the execution context stack and prevContext is the currently running execution context.
                    // f. Return undefined.

                    std::mem::swap(&mut context.realm.environments, environment);
                    std::mem::swap(&mut context.vm.stack, stack);
                    context.vm.push_frame(frame.clone());

                    context.vm.frame_mut().generator_resume_kind = GeneratorResumeKind::Throw;
                    context.vm.push(args.get_or_undefined(0).clone());
                    context.run();

                    *frame = context
                        .vm
                        .pop_frame()
                        .expect("generator call frame must exist");
                    std::mem::swap(&mut context.realm.environments, environment);
                    std::mem::swap(&mut context.vm.stack, stack);

                    Ok(JsValue::undefined())
                },
                Gc::new(GcRefCell::new((
                    context.realm.environments.clone(),
                    context.vm.stack.clone(),
                    context.vm.frame().clone(),
                ))),
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

        context.vm.push(JsValue::undefined());
        context.vm.frame_mut().early_return = Some(EarlyReturnType::Await);
        Ok(CompletionType::Return)
    }
}
