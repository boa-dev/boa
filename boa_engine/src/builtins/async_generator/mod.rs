//! This module implements the global `AsyncGenerator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-objects

use crate::{
    builtins::{
        generator::GeneratorContext, iterable::create_iter_result_object,
        promise::if_abrupt_reject_promise, promise::PromiseCapability, BuiltIn, JsArgs, Promise,
    },
    error::JsNativeError,
    object::{ConstructorBuilder, FunctionBuilder, JsObject, ObjectData},
    property::{Attribute, PropertyDescriptor},
    symbol::WellKnownSymbols,
    value::JsValue,
    vm::GeneratorResumeKind,
    Context, JsError, JsResult,
};
use boa_gc::{Finalize, Gc, GcCell, Trace};
use boa_profiler::Profiler;
use std::collections::VecDeque;

/// Indicates the state of an async generator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum AsyncGeneratorState {
    Undefined,
    SuspendedStart,
    SuspendedYield,
    Executing,
    AwaitingReturn,
    Completed,
}

/// `AsyncGeneratorRequest Records`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorrequest-records
#[derive(Debug, Clone, Finalize, Trace)]
pub(crate) struct AsyncGeneratorRequest {
    /// The `[[Completion]]` slot.
    pub(crate) completion: (JsResult<JsValue>, bool),

    /// The `[[Capability]]` slot.
    capability: PromiseCapability,
}

/// The internal representation on an `AsyncGenerator` object.
#[derive(Debug, Clone, Finalize, Trace)]
pub struct AsyncGenerator {
    /// The `[[AsyncGeneratorState]]` internal slot.
    #[unsafe_ignore_trace]
    pub(crate) state: AsyncGeneratorState,

    /// The `[[AsyncGeneratorContext]]` internal slot.
    pub(crate) context: Option<Gc<GcCell<GeneratorContext>>>,

    /// The `[[AsyncGeneratorQueue]]` internal slot.
    pub(crate) queue: VecDeque<AsyncGeneratorRequest>,
}

impl BuiltIn for AsyncGenerator {
    const NAME: &'static str = "AsyncGenerator";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let iterator_prototype = context
            .intrinsics()
            .objects()
            .iterator_prototypes()
            .async_iterator_prototype();

        let generator_function_prototype = context
            .intrinsics()
            .constructors()
            .async_generator_function()
            .prototype();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context
                .intrinsics()
                .constructors()
                .async_generator()
                .clone(),
        )
        .name(Self::NAME)
        .length(0)
        .property(
            WellKnownSymbols::to_string_tag(),
            Self::NAME,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .method(Self::next, "next", 1)
        .method(Self::r#return, "return", 1)
        .method(Self::throw, "throw", 1)
        .inherit(iterator_prototype)
        .build();

        context
            .intrinsics()
            .constructors()
            .async_generator()
            .prototype
            .insert_property(
                "constructor",
                PropertyDescriptor::builder()
                    .value(generator_function_prototype)
                    .writable(false)
                    .enumerable(false)
                    .configurable(true),
            );

        None
    }
}

impl AsyncGenerator {
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn constructor(
        _: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = context
            .intrinsics()
            .constructors()
            .async_generator()
            .prototype();

        let this = JsObject::from_proto_and_data(
            prototype,
            ObjectData::async_generator(Self {
                state: AsyncGeneratorState::Undefined,
                context: None,
                queue: VecDeque::new(),
            }),
        );

        Ok(this.into())
    }

    /// `AsyncGenerator.prototype.next ( value )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-prototype-next
    pub(crate) fn next(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let generator be the this value.
        let generator = this;

        // 2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let promise_capability = PromiseCapability::new(
            &context
                .intrinsics()
                .constructors()
                .promise()
                .constructor()
                .into(),
            context,
        )
        .expect("cannot fail with promise constructor");

        // 3. Let result be Completion(AsyncGeneratorValidate(generator, empty)).
        // 4. IfAbruptRejectPromise(result, promiseCapability).
        let generator_object: JsResult<_> = generator.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        if_abrupt_reject_promise!(generator_object, promise_capability, context);
        let mut generator_obj_mut = generator_object.borrow_mut();
        let generator: JsResult<_> = generator_obj_mut.as_async_generator_mut().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        if_abrupt_reject_promise!(generator, promise_capability, context);

        // 5. Let state be generator.[[AsyncGeneratorState]].
        let state = generator.state;

        // 6. If state is completed, then
        if state == AsyncGeneratorState::Completed {
            drop(generator_obj_mut);

            // a. Let iteratorResult be CreateIterResultObject(undefined, true).
            let iterator_result = create_iter_result_object(JsValue::undefined(), true, context);

            // b. Perform ! Call(promiseCapability.[[Resolve]], undefined, « iteratorResult »).
            promise_capability
                .resolve()
                .call(&JsValue::undefined(), &[iterator_result], context)
                .expect("cannot fail per spec");

            // c. Return promiseCapability.[[Promise]].
            return Ok(promise_capability.promise().clone().into());
        }

        // 7. Let completion be NormalCompletion(value).
        let completion = (Ok(args.get_or_undefined(0).clone()), false);

        // 8. Perform AsyncGeneratorEnqueue(generator, completion, promiseCapability).
        generator.enqueue(completion.clone(), promise_capability.clone());

        // 9. If state is either suspendedStart or suspendedYield, then
        if state == AsyncGeneratorState::SuspendedStart
            || state == AsyncGeneratorState::SuspendedYield
        {
            // a. Perform AsyncGeneratorResume(generator, completion).
            let generator_context = generator
                .context
                .clone()
                .expect("generator context cannot be empty here");

            drop(generator_obj_mut);

            Self::resume(
                generator_object,
                state,
                &generator_context,
                completion,
                context,
            );
        }

        // 11. Return promiseCapability.[[Promise]].
        Ok(promise_capability.promise().clone().into())
    }

    /// `AsyncGenerator.prototype.return ( value )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-prototype-return
    pub(crate) fn r#return(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let generator be the this value.
        let generator = this;

        // 2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let promise_capability = PromiseCapability::new(
            &context
                .intrinsics()
                .constructors()
                .promise()
                .constructor()
                .into(),
            context,
        )
        .expect("cannot fail with promise constructor");

        // 3. Let result be Completion(AsyncGeneratorValidate(generator, empty)).
        // 4. IfAbruptRejectPromise(result, promiseCapability).
        let generator_object: JsResult<_> = generator.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        if_abrupt_reject_promise!(generator_object, promise_capability, context);
        let mut generator_obj_mut = generator_object.borrow_mut();
        let generator: JsResult<_> = generator_obj_mut.as_async_generator_mut().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        if_abrupt_reject_promise!(generator, promise_capability, context);

        // 5. Let completion be Completion Record { [[Type]]: return, [[Value]]: value, [[Target]]: empty }.
        let completion = (Ok(args.get_or_undefined(0).clone()), true);

        // 6. Perform AsyncGeneratorEnqueue(generator, completion, promiseCapability).
        generator.enqueue(completion.clone(), promise_capability.clone());

        // 7. Let state be generator.[[AsyncGeneratorState]].
        let state = generator.state;

        // 8. If state is either suspendedStart or completed, then
        if state == AsyncGeneratorState::SuspendedStart || state == AsyncGeneratorState::Completed {
            // a. Set generator.[[AsyncGeneratorState]] to awaiting-return.
            generator.state = AsyncGeneratorState::AwaitingReturn;

            // b. Perform ! AsyncGeneratorAwaitReturn(generator).
            let next = generator
                .queue
                .front()
                .cloned()
                .expect("queue cannot be empty here");
            drop(generator_obj_mut);
            let (completion, _) = &next.completion;
            Self::await_return(generator_object.clone(), completion.clone(), context);
        }
        // 9. Else if state is suspendedYield, then
        else if state == AsyncGeneratorState::SuspendedYield {
            // a. Perform AsyncGeneratorResume(generator, completion).
            let generator_context = generator
                .context
                .clone()
                .expect("generator context cannot be empty here");

            drop(generator_obj_mut);
            Self::resume(
                generator_object,
                state,
                &generator_context,
                completion,
                context,
            );
        }

        // 11. Return promiseCapability.[[Promise]].
        Ok(promise_capability.promise().clone().into())
    }

    /// `AsyncGenerator.prototype.throw ( exception )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-prototype-throw
    pub(crate) fn throw(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let generator be the this value.
        let generator = this;

        // 2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let promise_capability = PromiseCapability::new(
            &context
                .intrinsics()
                .constructors()
                .promise()
                .constructor()
                .into(),
            context,
        )
        .expect("cannot fail with promise constructor");

        // 3. Let result be Completion(AsyncGeneratorValidate(generator, empty)).
        // 4. IfAbruptRejectPromise(result, promiseCapability).
        let generator_object: JsResult<_> = generator.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        if_abrupt_reject_promise!(generator_object, promise_capability, context);
        let mut generator_obj_mut = generator_object.borrow_mut();
        let generator: JsResult<_> = generator_obj_mut.as_async_generator_mut().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        if_abrupt_reject_promise!(generator, promise_capability, context);

        // 5. Let state be generator.[[AsyncGeneratorState]].
        let mut state = generator.state;

        // 6. If state is suspendedStart, then
        if state == AsyncGeneratorState::SuspendedStart {
            // a. Set generator.[[AsyncGeneratorState]] to completed.
            generator.state = AsyncGeneratorState::Completed;

            // b. Set state to completed.
            state = AsyncGeneratorState::Completed;
        }

        // 7. If state is completed, then
        if state == AsyncGeneratorState::Completed {
            drop(generator_obj_mut);

            // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « exception »).
            promise_capability
                .reject()
                .call(
                    &JsValue::undefined(),
                    &[args.get_or_undefined(0).clone()],
                    context,
                )
                .expect("cannot fail per spec");

            // b. Return promiseCapability.[[Promise]].
            return Ok(promise_capability.promise().clone().into());
        }

        // 8. Let completion be ThrowCompletion(exception).
        let completion = (
            Err(JsError::from_opaque(args.get_or_undefined(0).clone())),
            false,
        );

        // 9. Perform AsyncGeneratorEnqueue(generator, completion, promiseCapability).
        generator.enqueue(completion.clone(), promise_capability.clone());

        // 10. If state is suspendedYield, then
        if state == AsyncGeneratorState::SuspendedYield {
            let generator_context = generator
                .context
                .clone()
                .expect("generator context cannot be empty here");
            drop(generator_obj_mut);

            // a. Perform AsyncGeneratorResume(generator, completion).
            Self::resume(
                generator_object,
                state,
                &generator_context,
                completion,
                context,
            );
        }

        // 12. Return promiseCapability.[[Promise]].
        Ok(promise_capability.promise().clone().into())
    }

    /// `AsyncGeneratorEnqueue ( generator, completion, promiseCapability )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorenqueue
    pub(crate) fn enqueue(
        &mut self,
        completion: (JsResult<JsValue>, bool),
        promise_capability: PromiseCapability,
    ) {
        // 1. Let request be AsyncGeneratorRequest { [[Completion]]: completion, [[Capability]]: promiseCapability }.
        let request = AsyncGeneratorRequest {
            completion,
            capability: promise_capability,
        };

        // 2. Append request to the end of generator.[[AsyncGeneratorQueue]].
        self.queue.push_back(request);
    }

    /// `AsyncGeneratorCompleteStep ( generator, completion, done [ , realm ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorcompletestep
    pub(crate) fn complete_step(
        next: &AsyncGeneratorRequest,
        completion: JsResult<JsValue>,
        done: bool,
        context: &mut Context,
    ) {
        // 1. Let queue be generator.[[AsyncGeneratorQueue]].
        // 2. Assert: queue is not empty.
        // 3. Let next be the first element of queue.
        // 4. Remove the first element from queue.
        // 5. Let promiseCapability be next.[[Capability]].
        let promise_capability = &next.capability;

        // 6. Let value be completion.[[Value]].
        match completion {
            // 7. If completion.[[Type]] is throw, then
            Err(e) => {
                // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « value »).
                promise_capability
                    .reject()
                    .call(&JsValue::undefined(), &[e.to_opaque(context)], context)
                    .expect("cannot fail per spec");
            }
            // 8. Else,
            Ok(value) => {
                // a. Assert: completion.[[Type]] is normal.

                // TODO: Realm handling not implemented yet.
                // b. If realm is present, then
                // i. Let oldRealm be the running execution context's Realm.
                // ii. Set the running execution context's Realm to realm.
                // iii. Let iteratorResult be CreateIterResultObject(value, done).
                // iv. Set the running execution context's Realm to oldRealm.
                // c. Else,
                // i. Let iteratorResult be CreateIterResultObject(value, done).
                let iterator_result = create_iter_result_object(value, done, context);

                // d. Perform ! Call(promiseCapability.[[Resolve]], undefined, « iteratorResult »).
                promise_capability
                    .resolve()
                    .call(&JsValue::undefined(), &[iterator_result], context)
                    .expect("cannot fail per spec");
            }
        }
    }

    /// `AsyncGeneratorResume ( generator, completion )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorresume
    pub(crate) fn resume(
        generator: &JsObject,
        state: AsyncGeneratorState,
        generator_context: &Gc<GcCell<GeneratorContext>>,
        completion: (JsResult<JsValue>, bool),
        context: &mut Context,
    ) {
        // 1. Assert: generator.[[AsyncGeneratorState]] is either suspendedStart or suspendedYield.
        assert!(
            state == AsyncGeneratorState::SuspendedStart
                || state == AsyncGeneratorState::SuspendedYield
        );

        // 2. Let genContext be generator.[[AsyncGeneratorContext]].
        let mut generator_context_mut = generator_context.borrow_mut();

        // 3. Let callerContext be the running execution context.
        // 4. Suspend callerContext.

        // 5. Set generator.[[AsyncGeneratorState]] to executing.
        generator
            .borrow_mut()
            .as_async_generator_mut()
            .expect("already checked before")
            .state = AsyncGeneratorState::Executing;

        // 6. Push genContext onto the execution context stack; genContext is now the running execution context.
        std::mem::swap(
            &mut context.realm.environments,
            &mut generator_context_mut.environments,
        );
        std::mem::swap(&mut context.vm.stack, &mut generator_context_mut.stack);
        context
            .vm
            .push_frame(generator_context_mut.call_frame.clone());

        // 7. Resume the suspended evaluation of genContext using completion as the result of the operation that suspended it. Let result be the Completion Record returned by the resumed computation.
        match completion {
            (Ok(value), r#return) => {
                context.vm.push(value);
                if r#return {
                    context.vm.frame_mut().generator_resume_kind = GeneratorResumeKind::Return;
                } else {
                    context.vm.frame_mut().generator_resume_kind = GeneratorResumeKind::Normal;
                }
            }
            (Err(value), _) => {
                let value = value.to_opaque(context);
                context.vm.push(value);
                context.vm.frame_mut().generator_resume_kind = GeneratorResumeKind::Throw;
            }
        }
        drop(generator_context_mut);
        let result = context.run();

        let mut generator_context_mut = generator_context.borrow_mut();
        std::mem::swap(
            &mut context.realm.environments,
            &mut generator_context_mut.environments,
        );
        std::mem::swap(&mut context.vm.stack, &mut generator_context_mut.stack);
        generator_context_mut.call_frame =
            context.vm.pop_frame().expect("generator frame must exist");
        drop(generator_context_mut);

        // 8. Assert: result is never an abrupt completion.
        assert!(result.is_ok());

        // 9. Assert: When we return here, genContext has already been removed from the execution context stack and callerContext is the currently running execution context.
        // 10. Return unused.
    }

    /// `AsyncGeneratorAwaitReturn ( generator )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorawaitreturn
    pub(crate) fn await_return(
        generator: JsObject,
        completion: JsResult<JsValue>,
        context: &mut Context,
    ) {
        // 1. Let queue be generator.[[AsyncGeneratorQueue]].
        // 2. Assert: queue is not empty.
        // 3. Let next be the first element of queue.
        // 4. Let completion be Completion(next.[[Completion]]).

        // 5. Assert: completion.[[Type]] is return.
        let value = completion.expect("completion must be a return completion");

        // Note: The spec is currently broken here.
        // See: https://github.com/tc39/ecma262/pull/2683

        // 6. Let promise be ? PromiseResolve(%Promise%, completion.[[Value]]).
        let promise_completion = Promise::promise_resolve(
            context.intrinsics().constructors().promise().constructor(),
            value,
            context,
        );

        let promise = match promise_completion {
            Ok(value) => value,
            Err(value) => {
                let mut generator_borrow_mut = generator.borrow_mut();
                let gen = generator_borrow_mut
                    .as_async_generator_mut()
                    .expect("already checked before");
                gen.state = AsyncGeneratorState::Completed;
                let next = gen.queue.pop_front().expect("queue must not be empty");
                drop(generator_borrow_mut);
                Self::complete_step(&next, Err(value), true, context);
                Self::drain_queue(&generator, context);
                return;
            }
        };

        // 7. Let fulfilledClosure be a new Abstract Closure with parameters (value) that captures generator and performs the following steps when called:
        // 8. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 1, "", « »).
        let on_fulfilled = FunctionBuilder::closure_with_captures(
            context,
            |_this, args, generator, context| {
                let mut generator_borrow_mut = generator.borrow_mut();
                let gen = generator_borrow_mut
                    .as_async_generator_mut()
                    .expect("already checked before");

                // a. Set generator.[[AsyncGeneratorState]] to completed.
                gen.state = AsyncGeneratorState::Completed;

                // b. Let result be NormalCompletion(value).
                let result = Ok(args.get_or_undefined(0).clone());

                // c. Perform AsyncGeneratorCompleteStep(generator, result, true).
                let next = gen.queue.pop_front().expect("must have one entry");
                drop(generator_borrow_mut);
                Self::complete_step(&next, result, true, context);

                // d. Perform AsyncGeneratorDrainQueue(generator).
                Self::drain_queue(generator, context);

                // e. Return undefined.
                Ok(JsValue::undefined())
            },
            generator.clone(),
        )
        .name("")
        .length(1)
        .build();

        // 9. Let rejectedClosure be a new Abstract Closure with parameters (reason) that captures generator and performs the following steps when called:
        // 10. Let onRejected be CreateBuiltinFunction(rejectedClosure, 1, "", « »).
        let on_rejected = FunctionBuilder::closure_with_captures(
            context,
            |_this, args, generator, context| {
                let mut generator_borrow_mut = generator.borrow_mut();
                let gen = generator_borrow_mut
                    .as_async_generator_mut()
                    .expect("already checked before");

                // a. Set generator.[[AsyncGeneratorState]] to completed.
                gen.state = AsyncGeneratorState::Completed;

                // b. Let result be ThrowCompletion(reason).
                let result = Err(JsError::from_opaque(args.get_or_undefined(0).clone()));

                // c. Perform AsyncGeneratorCompleteStep(generator, result, true).
                let next = gen.queue.pop_front().expect("must have one entry");
                drop(generator_borrow_mut);
                Self::complete_step(&next, result, true, context);

                // d. Perform AsyncGeneratorDrainQueue(generator).
                Self::drain_queue(generator, context);

                // e. Return undefined.
                Ok(JsValue::undefined())
            },
            generator,
        )
        .name("")
        .length(1)
        .build();

        // 11. Perform PerformPromiseThen(promise, onFulfilled, onRejected).
        let promise_obj = promise
            .as_object()
            .expect("constructed promise must be a promise");
        promise_obj
            .borrow_mut()
            .as_promise_mut()
            .expect("constructed promise must be a promise")
            .perform_promise_then(&on_fulfilled.into(), &on_rejected.into(), None, context);
    }

    /// `AsyncGeneratorDrainQueue ( generator )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratordrainqueue
    pub(crate) fn drain_queue(generator: &JsObject, context: &mut Context) {
        let mut generator_borrow_mut = generator.borrow_mut();
        let gen = generator_borrow_mut
            .as_async_generator_mut()
            .expect("already checked before");

        // 1. Assert: generator.[[AsyncGeneratorState]] is completed.
        assert_eq!(gen.state, AsyncGeneratorState::Completed);

        // 2. Let queue be generator.[[AsyncGeneratorQueue]].
        let queue = &mut gen.queue;

        // 3. If queue is empty, return unused.
        if queue.is_empty() {
            return;
        }

        // 4. Let done be false.
        // 5. Repeat, while done is false,
        loop {
            // a. Let next be the first element of queue.
            let next = queue.front().expect("must have entry");

            // b. Let completion be Completion(next.[[Completion]]).
            match &next.completion {
                // c. If completion.[[Type]] is return, then
                (completion, true) => {
                    // i. Set generator.[[AsyncGeneratorState]] to awaiting-return.
                    gen.state = AsyncGeneratorState::AwaitingReturn;

                    // ii. Perform ! AsyncGeneratorAwaitReturn(generator).
                    let completion = completion.clone();
                    drop(generator_borrow_mut);
                    Self::await_return(generator.clone(), completion, context);

                    // iii. Set done to true.
                    break;
                }
                // d. Else,
                (completion, false) => {
                    // i. If completion.[[Type]] is normal, then
                    let completion = if completion.is_ok() {
                        // 1. Set completion to NormalCompletion(undefined).
                        Ok(JsValue::undefined())
                    } else {
                        completion.clone()
                    };

                    // ii. Perform AsyncGeneratorCompleteStep(generator, completion, true).
                    let next = queue.pop_front().expect("must have entry");
                    Self::complete_step(&next, completion, true, context);

                    // iii. If queue is empty, set done to true.
                    if queue.is_empty() {
                        break;
                    }
                }
            }
        }
    }
}
