//! Boa's implementation of ECMAScript's global `AsyncGenerator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-objects

use crate::{
    builtins::{
        generator::GeneratorContext, iterable::create_iter_result_object,
        promise::if_abrupt_reject_promise, promise::PromiseCapability, Promise,
    },
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject, CONSTRUCTOR},
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
    value::JsValue,
    vm::{CompletionRecord, GeneratorResumeKind},
    Context, JsArgs, JsError, JsResult,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use std::collections::VecDeque;

use super::{BuiltInBuilder, IntrinsicObject};

/// Indicates the state of an async generator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum AsyncGeneratorState {
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
    pub(crate) completion: CompletionRecord,

    /// The `[[Capability]]` slot.
    capability: PromiseCapability,
}

/// The internal representation of an `AsyncGenerator` object.
#[derive(Debug, Clone, Finalize, Trace)]
pub struct AsyncGenerator {
    /// The `[[AsyncGeneratorState]]` internal slot.
    #[unsafe_ignore_trace]
    pub(crate) state: AsyncGeneratorState,

    /// The `[[AsyncGeneratorContext]]` internal slot.
    pub(crate) context: Option<GeneratorContext>,

    /// The `[[AsyncGeneratorQueue]]` internal slot.
    pub(crate) queue: VecDeque<AsyncGeneratorRequest>,
}

impl IntrinsicObject for AsyncGenerator {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .async_iterator(),
            )
            .static_method(Self::next, "next", 1)
            .static_method(Self::r#return, "return", 1)
            .static_method(Self::throw, "throw", 1)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .static_property(
                CONSTRUCTOR,
                realm
                    .intrinsics()
                    .constructors()
                    .async_generator_function()
                    .prototype(),
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().async_generator()
    }
}

impl AsyncGenerator {
    const NAME: &'static str = "AsyncGenerator";

    /// `AsyncGenerator.prototype.next ( value )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-prototype-next
    pub(crate) fn next(
        this: &JsValue,
        args: &[JsValue],
        context: &mut dyn Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let generator be the this value.
        let generator = this;

        // 2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let promise_capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
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
        let completion = CompletionRecord::Normal(args.get_or_undefined(0).clone());

        // 8. Perform AsyncGeneratorEnqueue(generator, completion, promiseCapability).
        generator.enqueue(completion.clone(), promise_capability.clone());

        // 9. If state is either suspendedStart or suspendedYield, then
        if state == AsyncGeneratorState::SuspendedStart
            || state == AsyncGeneratorState::SuspendedYield
        {
            // a. Perform AsyncGeneratorResume(generator, completion).
            let generator_context = generator
                .context
                .take()
                .expect("generator context cannot be empty here");

            drop(generator_obj_mut);

            Self::resume(
                generator_object,
                state,
                generator_context,
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
        context: &mut dyn Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let generator be the this value.
        let generator = this;

        // 2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let promise_capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
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
        let return_value = args.get_or_undefined(0).clone();
        let completion = CompletionRecord::Return(return_value.clone());

        // 6. Perform AsyncGeneratorEnqueue(generator, completion, promiseCapability).
        generator.enqueue(completion.clone(), promise_capability.clone());

        // 7. Let state be generator.[[AsyncGeneratorState]].
        let state = generator.state;

        // 8. If state is either suspendedStart or completed, then
        if state == AsyncGeneratorState::SuspendedStart || state == AsyncGeneratorState::Completed {
            // a. Set generator.[[AsyncGeneratorState]] to awaiting-return.
            generator.state = AsyncGeneratorState::AwaitingReturn;

            // b. Perform ! AsyncGeneratorAwaitReturn(generator).
            drop(generator_obj_mut);
            Self::await_return(generator_object.clone(), return_value, context);
        }
        // 9. Else if state is suspendedYield, then
        else if state == AsyncGeneratorState::SuspendedYield {
            // a. Perform AsyncGeneratorResume(generator, completion).
            let generator_context = generator
                .context
                .take()
                .expect("generator context cannot be empty here");

            drop(generator_obj_mut);
            Self::resume(
                generator_object,
                state,
                generator_context,
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
        context: &mut dyn Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let generator be the this value.
        let generator = this;

        // 2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let promise_capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
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
            generator.context = None;

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
        let completion =
            CompletionRecord::Throw(JsError::from_opaque(args.get_or_undefined(0).clone()));

        // 9. Perform AsyncGeneratorEnqueue(generator, completion, promiseCapability).
        generator.enqueue(completion.clone(), promise_capability.clone());

        // 10. If state is suspendedYield, then
        if state == AsyncGeneratorState::SuspendedYield {
            let generator_context = generator
                .context
                .take()
                .expect("generator context cannot be empty here");
            drop(generator_obj_mut);

            // a. Perform AsyncGeneratorResume(generator, completion).
            Self::resume(
                generator_object,
                state,
                generator_context,
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
        completion: CompletionRecord,
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
        realm: Option<Realm>,
        context: &mut dyn Context<'_>,
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

                // b. If realm is present, then
                let iterator_result = if let Some(realm) = realm {
                    // i. Let oldRealm be the running execution context's Realm.
                    // ii. Set the running execution context's Realm to realm.
                    let old_realm = context.enter_realm(realm);

                    // iii. Let iteratorResult be CreateIterResultObject(value, done).
                    let iterator_result = create_iter_result_object(value, done, context);

                    // iv. Set the running execution context's Realm to oldRealm.
                    context.enter_realm(old_realm);

                    iterator_result
                } else {
                    // c. Else,
                    // i. Let iteratorResult be CreateIterResultObject(value, done).
                    create_iter_result_object(value, done, context)
                };

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
        mut generator_context: GeneratorContext,
        completion: CompletionRecord,
        context: &mut dyn Context<'_>,
    ) {
        // 1. Assert: generator.[[AsyncGeneratorState]] is either suspendedStart or suspendedYield.
        assert!(
            state == AsyncGeneratorState::SuspendedStart
                || state == AsyncGeneratorState::SuspendedYield
        );

        // 2. Let genContext be generator.[[AsyncGeneratorContext]].

        // 3. Let callerContext be the running execution context.
        // 4. Suspend callerContext.

        // 5. Set generator.[[AsyncGeneratorState]] to executing.
        generator
            .borrow_mut()
            .as_async_generator_mut()
            .expect("already checked before")
            .state = AsyncGeneratorState::Executing;

        let (value, resume_kind) = match completion {
            CompletionRecord::Normal(val) => (val, GeneratorResumeKind::Normal),
            CompletionRecord::Return(val) => (val, GeneratorResumeKind::Return),
            CompletionRecord::Throw(err) => (err.to_opaque(context), GeneratorResumeKind::Throw),
        };
        // 6. Push genContext onto the execution context stack; genContext is now the running execution context.

        let result = generator_context.resume(Some(value), resume_kind, context);

        // 7. Resume the suspended evaluation of genContext using completion as the result of the operation that suspended it. Let result be the Completion Record returned by the resumed computation.

        generator
            .borrow_mut()
            .as_async_generator_mut()
            .expect("already checked before")
            .context = Some(generator_context);

        // 8. Assert: result is never an abrupt completion.
        assert!(!result.is_throw_completion());

        // 9. Assert: When we return here, genContext has already been removed from the execution context stack and
        // callerContext is the currently running execution context.
        // 10. Return unused.
    }

    /// `AsyncGeneratorAwaitReturn ( generator )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorawaitreturn
    pub(crate) fn await_return(generator: JsObject, value: JsValue, context: &mut dyn Context<'_>) {
        // 1. Let queue be generator.[[AsyncGeneratorQueue]].
        // 2. Assert: queue is not empty.
        // 3. Let next be the first element of queue.
        // 4. Let completion be Completion(next.[[Completion]]).

        // Note: The spec is currently broken here.
        // See: https://github.com/tc39/ecma262/pull/2683

        // 6. Let promise be ? PromiseResolve(%Promise%, completion.[[Value]]).
        let promise_completion = Promise::promise_resolve(
            &context.intrinsics().constructors().promise().constructor(),
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
                gen.context = None;
                let next = gen.queue.pop_front().expect("queue must not be empty");
                drop(generator_borrow_mut);
                Self::complete_step(&next, Err(value), true, None, context);
                Self::drain_queue(&generator, context);
                return;
            }
        };

        // 7. Let fulfilledClosure be a new Abstract Closure with parameters (value) that captures generator and performs the following steps when called:
        // 8. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 1, "", « »).
        let on_fulfilled = FunctionObjectBuilder::new(
            context,
            NativeFunction::from_copy_closure_with_captures(
                |_this, args, generator, context| {
                    let next = {
                        let mut generator_borrow_mut = generator.borrow_mut();
                        let gen = generator_borrow_mut
                            .as_async_generator_mut()
                            .expect("already checked before");

                        // a. Set generator.[[AsyncGeneratorState]] to completed.
                        gen.state = AsyncGeneratorState::Completed;
                        gen.context = None;

                        gen.queue.pop_front().expect("must have one entry")
                    };

                    // b. Let result be NormalCompletion(value).
                    let result = Ok(args.get_or_undefined(0).clone());

                    // c. Perform AsyncGeneratorCompleteStep(generator, result, true).
                    Self::complete_step(&next, result, true, None, context);

                    // d. Perform AsyncGeneratorDrainQueue(generator).
                    Self::drain_queue(generator, context);

                    // e. Return undefined.
                    Ok(JsValue::undefined())
                },
                generator.clone(),
            ),
        )
        .name("")
        .length(1)
        .build();

        // 9. Let rejectedClosure be a new Abstract Closure with parameters (reason) that captures generator and performs the following steps when called:
        // 10. Let onRejected be CreateBuiltinFunction(rejectedClosure, 1, "", « »).
        let on_rejected = FunctionObjectBuilder::new(
            context,
            NativeFunction::from_copy_closure_with_captures(
                |_this, args, generator, context| {
                    let mut generator_borrow_mut = generator.borrow_mut();
                    let gen = generator_borrow_mut
                        .as_async_generator_mut()
                        .expect("already checked before");

                    // a. Set generator.[[AsyncGeneratorState]] to completed.
                    gen.state = AsyncGeneratorState::Completed;
                    gen.context = None;

                    // b. Let result be ThrowCompletion(reason).
                    let result = Err(JsError::from_opaque(args.get_or_undefined(0).clone()));

                    // c. Perform AsyncGeneratorCompleteStep(generator, result, true).
                    let next = gen.queue.pop_front().expect("must have one entry");
                    drop(generator_borrow_mut);
                    Self::complete_step(&next, result, true, None, context);

                    // d. Perform AsyncGeneratorDrainQueue(generator).
                    Self::drain_queue(generator, context);

                    // e. Return undefined.
                    Ok(JsValue::undefined())
                },
                generator,
            ),
        )
        .name("")
        .length(1)
        .build();

        // 11. Perform PerformPromiseThen(promise, onFulfilled, onRejected).
        Promise::perform_promise_then(
            &promise,
            Some(on_fulfilled),
            Some(on_rejected),
            None,
            context,
        );
    }

    /// `AsyncGeneratorDrainQueue ( generator )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratordrainqueue
    pub(crate) fn drain_queue(generator: &JsObject, context: &mut dyn Context<'_>) {
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
            match next.completion.clone() {
                // c. If completion.[[Type]] is return, then
                CompletionRecord::Return(val) => {
                    // i. Set generator.[[AsyncGeneratorState]] to awaiting-return.
                    gen.state = AsyncGeneratorState::AwaitingReturn;
                    drop(generator_borrow_mut);

                    // ii. Perform ! AsyncGeneratorAwaitReturn(generator).
                    Self::await_return(generator.clone(), val, context);

                    // iii. Set done to true.
                    break;
                }
                // d. Else,
                completion => {
                    // i. If completion.[[Type]] is normal, then
                    // 1. Set completion to NormalCompletion(undefined).
                    let completion = completion.consume().map(|_| JsValue::undefined());

                    // ii. Perform AsyncGeneratorCompleteStep(generator, completion, true).
                    let next = queue.pop_front().expect("must have entry");
                    Self::complete_step(&next, completion, true, None, context);

                    // iii. If queue is empty, set done to true.
                    if queue.is_empty() {
                        break;
                    }
                }
            }
        }
    }
}
