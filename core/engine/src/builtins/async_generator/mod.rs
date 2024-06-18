//! Boa's implementation of ECMAScript's global `AsyncGenerator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-objects

use crate::{
    builtins::{
        generator::GeneratorContext,
        iterable::create_iter_result_object,
        promise::{if_abrupt_reject_promise, PromiseCapability},
        Promise,
    },
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject, CONSTRUCTOR},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
    value::JsValue,
    vm::{CompletionRecord, GeneratorResumeKind},
    Context, JsArgs, JsData, JsError, JsResult, JsString,
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
#[derive(Debug, Clone, Finalize, Trace, JsData)]
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
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .async_iterator(),
            )
            .static_method(Self::next, js_string!("next"), 1)
            .static_method(Self::r#return, js_string!("return"), 1)
            .static_method(Self::throw, js_string!("throw"), 1)
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
    const NAME: JsString = StaticJsStrings::ASYNC_GENERATOR;

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
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail with promise constructor");

        // 3. Let result be Completion(AsyncGeneratorValidate(generator, empty)).
        // 4. IfAbruptRejectPromise(result, promiseCapability).
        let result: JsResult<_> = generator.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        let generator = if_abrupt_reject_promise!(result, promise_capability, context);
        let result: JsResult<_> = generator.clone().downcast::<Self>().map_err(|_| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        let generator = if_abrupt_reject_promise!(result, promise_capability, context);

        // 5. Let state be generator.[[AsyncGeneratorState]].
        let state = generator.borrow().data.state;

        // 6. If state is completed, then
        if state == AsyncGeneratorState::Completed {
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
        Self::enqueue(&generator, completion, promise_capability.clone(), context);

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
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail with promise constructor");

        // 3. Let result be Completion(AsyncGeneratorValidate(generator, empty)).
        // 4. IfAbruptRejectPromise(result, promiseCapability).
        let result: JsResult<_> = generator.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        let generator_object = if_abrupt_reject_promise!(result, promise_capability, context);
        let result: JsResult<_> = generator_object.clone().downcast::<Self>().map_err(|_| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        let generator = if_abrupt_reject_promise!(result, promise_capability, context);

        // 5. Let completion be Completion Record { [[Type]]: return, [[Value]]: value, [[Target]]: empty }.
        let return_value = args.get_or_undefined(0).clone();
        let completion = CompletionRecord::Return(return_value);

        // 6. Perform AsyncGeneratorEnqueue(generator, completion, promiseCapability).
        Self::enqueue(&generator, completion, promise_capability.clone(), context);

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
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail with promise constructor");

        // 3. Let result be Completion(AsyncGeneratorValidate(generator, empty)).
        // 4. IfAbruptRejectPromise(result, promiseCapability).
        let result: JsResult<_> = generator.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        let generator_object = if_abrupt_reject_promise!(result, promise_capability, context);
        let result: JsResult<_> = generator_object.clone().downcast::<Self>().map_err(|_| {
            JsNativeError::typ()
                .with_message("generator resumed on non generator object")
                .into()
        });
        let generator = if_abrupt_reject_promise!(result, promise_capability, context);
        let mut gen = generator.borrow_mut();

        // 5. Let state be generator.[[AsyncGeneratorState]].
        let mut state = gen.data.state;

        // 6. If state is suspendedStart, then
        if state == AsyncGeneratorState::SuspendedStart {
            // a. Set generator.[[AsyncGeneratorState]] to completed.
            gen.data.state = AsyncGeneratorState::Completed;
            gen.data.context = None;

            // b. Set state to completed.
            state = AsyncGeneratorState::Completed;
        }

        drop(gen);

        // 7. If state is completed, then
        if state == AsyncGeneratorState::Completed {
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
        Self::enqueue(
            &generator,
            completion.clone(),
            promise_capability.clone(),
            context,
        );

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
        generator: &JsObject<AsyncGenerator>,
        completion: CompletionRecord,
        promise_capability: PromiseCapability,
        context: &mut Context,
    ) {
        let mut gen = generator.borrow_mut();
        // 1. Let request be AsyncGeneratorRequest { [[Completion]]: completion, [[Capability]]: promiseCapability }.
        let request = AsyncGeneratorRequest {
            completion,
            capability: promise_capability,
        };

        // 2. Append request to the end of generator.[[AsyncGeneratorQueue]].
        gen.data.queue.push_back(request);

        // Patch that mirrors https://262.ecma-international.org/12.0/#sec-asyncgeneratorenqueue
        // This resolves the return bug.
        if gen.data.state != AsyncGeneratorState::Executing {
            drop(gen);
            AsyncGenerator::resume_next(generator, context);
        }
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

    /// `AsyncGeneratorAwaitReturn ( generator )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorawaitreturn
    pub(crate) fn await_return(
        generator: JsObject<AsyncGenerator>,
        value: JsValue,
        context: &mut Context,
    ) {
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
                let next = {
                    let mut gen = generator.borrow_mut();
                    gen.data.state = AsyncGeneratorState::Completed;
                    gen.data.context = None;
                    gen.data.queue.pop_front().expect("queue must not be empty")
                };
                Self::complete_step(&next, Err(value), true, None, context);
                Self::resume_next(&generator, context);
                return;
            }
        };

        // 7. Let fulfilledClosure be a new Abstract Closure with parameters (value) that captures generator and performs the following steps when called:
        // 8. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 1, "", « »).
        let on_fulfilled = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_copy_closure_with_captures(
                |_this, args, generator, context| {
                    let next = {
                        let mut gen = generator.borrow_mut();

                        // a. Set generator.[[AsyncGeneratorState]] to completed.
                        gen.data.state = AsyncGeneratorState::Completed;
                        gen.data.context = None;

                        gen.data.queue.pop_front().expect("must have one entry")
                    };

                    // b. Let result be NormalCompletion(value).
                    let result = Ok(args.get_or_undefined(0).clone());

                    // c. Perform AsyncGeneratorCompleteStep(generator, result, true).
                    Self::complete_step(&next, result, true, None, context);

                    // d. Perform AsyncGeneratorDrainQueue(generator).
                    Self::resume_next(generator, context);

                    // e. Return undefined.
                    Ok(JsValue::undefined())
                },
                generator.clone(),
            ),
        )
        .name(js_string!(""))
        .length(1)
        .build();

        // 9. Let rejectedClosure be a new Abstract Closure with parameters (reason) that captures generator and performs the following steps when called:
        // 10. Let onRejected be CreateBuiltinFunction(rejectedClosure, 1, "", « »).
        let on_rejected = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_copy_closure_with_captures(
                |_this, args, generator, context| {
                    let next = {
                        let mut gen = generator.borrow_mut();

                        // a. Set generator.[[AsyncGeneratorState]] to completed.
                        gen.data.state = AsyncGeneratorState::Completed;
                        gen.data.context = None;

                        gen.data.queue.pop_front().expect("must have one entry")
                    };

                    // b. Let result be ThrowCompletion(reason).
                    let result = Err(JsError::from_opaque(args.get_or_undefined(0).clone()));

                    // c. Perform AsyncGeneratorCompleteStep(generator, result, true).
                    Self::complete_step(&next, result, true, None, context);

                    // d. Perform AsyncGeneratorDrainQueue(generator).
                    Self::resume_next(generator, context);

                    // e. Return undefined.
                    Ok(JsValue::undefined())
                },
                generator,
            ),
        )
        .name(js_string!(""))
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

    /// [`AsyncGeneratorResumeNext ( generator )`][spec]
    ///
    /// [spec]: https://262.ecma-international.org/12.0/#sec-asyncgeneratorresumenext
    pub(crate) fn resume_next(generator: &JsObject<AsyncGenerator>, context: &mut Context) {
        // 1. Assert: generator is an AsyncGenerator instance.
        let mut gen = generator.borrow_mut();
        // 2. Let state be generator.[[AsyncGeneratorState]].
        match gen.data.state {
            // 3. Assert: state is not executing.
            AsyncGeneratorState::Executing => panic!("3. Assert: state is not executing."),
            // 4. If state is awaiting-return, return undefined.
            AsyncGeneratorState::AwaitingReturn => return,
            _ => {}
        }

        // 5. Let queue be generator.[[AsyncGeneratorQueue]].
        // 6. If queue is an empty List, return undefined.
        // 7. Let next be the value of the first element of queue.
        // 8. Assert: next is an AsyncGeneratorRequest record.
        let Some(next) = gen.data.queue.front() else {
            return;
        };
        // 9. Let completion be next.[[Completion]].
        let completion = &next.completion;

        match (completion, gen.data.state) {
            // 11. Else if state is completed, return ! AsyncGeneratorResolve(generator, undefined, true).
            (CompletionRecord::Normal(_), s) => {
                if s == AsyncGeneratorState::Completed {
                    let next = gen
                        .data
                        .queue
                        .pop_front()
                        .expect("already have a reference to the front");
                    drop(gen);
                    AsyncGenerator::complete_step(
                        &next,
                        Ok(JsValue::undefined()),
                        true,
                        None,
                        context,
                    );
                    return AsyncGenerator::resume_next(generator, context);
                }
            }
            // b. If state is completed, then
            //    i. If completion.[[Type]] is return, then
            (
                CompletionRecord::Return(val),
                AsyncGeneratorState::SuspendedStart | AsyncGeneratorState::Completed,
            ) => {
                let val = val.clone();
                // 1. Set generator.[[AsyncGeneratorState]] to awaiting-return.
                gen.data.state = AsyncGeneratorState::AwaitingReturn;
                drop(gen);

                // Steps 2-11 are superseeded by `AsyncGeneratorAwaitReturn`
                AsyncGenerator::await_return(generator.clone(), val, context);

                // 12. Return undefined.
                return;
            }
            // ii. Else,
            (
                CompletionRecord::Throw(e),
                AsyncGeneratorState::SuspendedStart | AsyncGeneratorState::Completed,
            ) => {
                let e = e.clone();
                // 1. Assert: completion.[[Type]] is throw.
                // 2. Perform ! AsyncGeneratorReject(generator, completion.[[Value]]).
                gen.data.state = AsyncGeneratorState::Completed;

                let next = gen
                    .data
                    .queue
                    .pop_front()
                    .expect("already have a reference to the front");
                drop(gen);
                AsyncGenerator::complete_step(&next, Err(e), true, None, context);
                // 3. Return undefined.
                return AsyncGenerator::resume_next(generator, context);
            }
            _ => {}
        }

        // 12. Assert: state is either suspendedStart or suspendedYield.
        assert!(matches!(
            gen.data.state,
            AsyncGeneratorState::SuspendedStart | AsyncGeneratorState::SuspendedYield
        ));

        let completion = completion.clone();

        // 16. Set generator.[[AsyncGeneratorState]] to executing.
        gen.data.state = AsyncGeneratorState::Executing;

        // 13. Let genContext be generator.[[AsyncGeneratorContext]].
        let mut generator_context = gen
            .data
            .context
            .take()
            .expect("generator context cannot be empty here");

        drop(gen);

        let (value, resume_kind) = match completion {
            CompletionRecord::Normal(val) => (val, GeneratorResumeKind::Normal),
            CompletionRecord::Return(val) => (val, GeneratorResumeKind::Return),
            CompletionRecord::Throw(err) => (err.to_opaque(context), GeneratorResumeKind::Throw),
        };

        // 14. Let callerContext be the running execution context.
        // 15. Suspend callerContext.
        // 17. Push genContext onto the execution context stack; genContext is now the running execution context.
        // 18. Resume the suspended evaluation of genContext using completion as the result of the operation that suspended it.
        //     Let result be the completion record returned by the resumed computation.
        let result = generator_context.resume(Some(value), resume_kind, context);

        // 19. Assert: result is never an abrupt completion.
        assert!(!matches!(result, CompletionRecord::Throw(_)));

        generator.borrow_mut().data.context = Some(generator_context);

        // 20. Assert: When we return here, genContext has already been removed from the execution context stack and
        //     callerContext is the currently running execution context.
        // 21. Return undefined.
    }
}
