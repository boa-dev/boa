//! This module implements the global `Promise` object.

#![allow(dead_code, unused_results, unused_variables)]

#[cfg(test)]
mod tests;

mod promise_job;

use boa_gc::{Finalize, Gc, Trace};
use boa_profiler::Profiler;
use tap::{Conv, Pipe};

use crate::{
    builtins::BuiltIn,
    context::intrinsics::StandardConstructors,
    job::JobCallback,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        JsObject, ObjectData,
    },
    property::Attribute,
    value::JsValue,
    Context, JsResult,
};

use self::promise_job::PromiseJob;

use super::JsArgs;

/// JavaScript `Array` built-in implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Array;

#[derive(Debug, Clone, Trace, Finalize)]
enum PromiseState {
    Pending,
    Fulfilled,
    Rejected,
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Promise {
    promise_result: Option<JsValue>,
    promise_state: PromiseState,
    promise_fulfill_reactions: Vec<ReactionRecord>,
    promise_reject_reactions: Vec<ReactionRecord>,
    promise_is_handled: bool,
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct ReactionRecord {
    promise_capability: Option<PromiseCapability>,
    reaction_type: ReactionType,
    handler: Option<JobCallback>,
}

#[derive(Debug, Clone, Trace, Finalize)]
enum ReactionType {
    Fulfill,
    Reject,
}

#[derive(Debug, Clone, Trace, Finalize)]
struct PromiseCapability {
    promise: JsValue,
    resolve: JsValue,
    reject: JsValue,
}

#[derive(Debug, Trace, Finalize)]
struct PromiseCapabilityCaptures {
    promise_capability: Gc<boa_gc::Cell<PromiseCapability>>,
}

impl PromiseCapability {
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-newpromisecapability
    fn new(c: &JsValue, context: &mut Context) -> JsResult<Self> {
        match c.as_constructor() {
            // 1. If IsConstructor(C) is false, throw a TypeError exception.
            None => context.throw_type_error("PromiseCapability: expected constructor"),
            Some(c) => {
                let c = c.clone();

                // 2. NOTE: C is assumed to be a constructor function that supports the parameter conventions of the Promise constructor (see 27.2.3.1).
                // 3. Let promiseCapability be the PromiseCapability Record { [[Promise]]: undefined, [[Resolve]]: undefined, [[Reject]]: undefined }.
                let promise_capability = Gc::new(boa_gc::Cell::new(Self {
                    promise: JsValue::Undefined,
                    reject: JsValue::Undefined,
                    resolve: JsValue::Undefined,
                }));

                // 4. Let executorClosure be a new Abstract Closure with parameters (resolve, reject) that captures promiseCapability and performs the following steps when called:
                // 5. Let executor be CreateBuiltinFunction(executorClosure, 2, "", « »).
                let executor = FunctionBuilder::closure_with_captures(
                    context,
                    |this, args: &[JsValue], captures: &mut PromiseCapabilityCaptures, context| {
                        let promise_capability: &mut Self =
                            &mut captures.promise_capability.try_borrow_mut().expect("msg");

                        // a. If promiseCapability.[[Resolve]] is not undefined, throw a TypeError exception.
                        if !promise_capability.resolve.is_undefined() {
                            return context.throw_type_error(
                                "promiseCapability.[[Resolve]] is not undefined",
                            );
                        }

                        // b. If promiseCapability.[[Reject]] is not undefined, throw a TypeError exception.
                        if !promise_capability.reject.is_undefined() {
                            return context
                                .throw_type_error("promiseCapability.[[Reject]] is not undefined");
                        }

                        let resolve = args.get_or_undefined(0);
                        let reject = args.get_or_undefined(1);

                        // c. Set promiseCapability.[[Resolve]] to resolve.
                        promise_capability.resolve = resolve.clone();

                        // d. Set promiseCapability.[[Reject]] to reject.
                        promise_capability.reject = reject.clone();

                        // e. Return undefined.
                        Ok(JsValue::Undefined)
                    },
                    PromiseCapabilityCaptures {
                        promise_capability: promise_capability.clone(),
                    },
                )
                .name("")
                .length(2)
                .build()
                .into();

                // 6. Let promise be ? Construct(C, « executor »).
                let promise = c.construct(&[executor], &c.clone().into(), context)?;

                let promise_capability: &mut Self =
                    &mut promise_capability.try_borrow_mut().expect("msg");

                let resolve = promise_capability.resolve.clone();
                let reject = promise_capability.reject.clone();

                // 7. If IsCallable(promiseCapability.[[Resolve]]) is false, throw a TypeError exception.
                if !resolve.is_callable() {
                    return context
                        .throw_type_error("promiseCapability.[[Resolve]] is not callable");
                }

                // 8. If IsCallable(promiseCapability.[[Reject]]) is false, throw a TypeError exception.
                if !reject.is_callable() {
                    return context
                        .throw_type_error("promiseCapability.[[Reject]] is not callable");
                }

                // 9. Set promiseCapability.[[Promise]] to promise.
                promise_capability.reject = promise;

                // 10. Return promiseCapability.
                Ok(promise_capability.clone())
            }
        }
    }
}

impl BuiltIn for Promise {
    const NAME: &'static str = "Promise";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().promise().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .method(Self::then, "then", 1)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

struct ResolvedRecord {
    value: bool,
}

struct ResolvingFunctionsRecord {
    resolve: JsValue,
    reject: JsValue,
}

#[derive(Debug, Trace, Finalize)]
struct RejectResolveCaptures {
    promise: JsObject,
    already_resolved: JsObject,
}

impl Promise {
    const LENGTH: usize = 1;

    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise-executor
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return context.throw_type_error("Promise NewTarget cannot be undefined");
        }

        let executor = args.get_or_undefined(0);

        // 2. If IsCallable(executor) is false, throw a TypeError exception.
        if !executor.is_callable() {
            return context.throw_type_error("Promise executor is not callable");
        }

        // 3. Let promise be ? OrdinaryCreateFromConstructor(NewTarget, "%Promise.prototype%", « [[PromiseState]], [[PromiseResult]], [[PromiseFulfillReactions]], [[PromiseRejectReactions]], [[PromiseIsHandled]] »).
        let promise =
            get_prototype_from_constructor(new_target, StandardConstructors::promise, context)?;

        let promise = JsObject::from_proto_and_data(
            promise,
            ObjectData::promise(Self {
                promise_result: None,
                // 4. Set promise.[[PromiseState]] to pending.
                promise_state: PromiseState::Pending,
                // 5. Set promise.[[PromiseFulfillReactions]] to a new empty List.
                promise_fulfill_reactions: vec![],
                // 6. Set promise.[[PromiseRejectReactions]] to a new empty List.
                promise_reject_reactions: vec![],
                // 7. Set promise.[[PromiseIsHandled]] to false.
                promise_is_handled: false,
            }),
        );

        // // 8. Let resolvingFunctions be CreateResolvingFunctions(promise).
        let resolving_functions = Self::create_resolving_functions(&promise, context)?;

        // // 9. Let completion Completion(Call(executor, undefined, « resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]] »)be ).
        let completion = context.call(
            executor,
            &JsValue::Undefined,
            &[
                resolving_functions.resolve,
                resolving_functions.reject.clone(),
            ],
        );

        // 10. If completion is an abrupt completion, then
        if let Err(value) = completion {
            // a. Perform ? Call(resolvingFunctions.[[Reject]], undefined, « completion.[[Value]] »).
            let _reject_result =
                context.call(&resolving_functions.reject, &JsValue::Undefined, &[value]);
        }

        // 11. Return promise.
        promise.conv::<JsValue>().pipe(Ok)
    }

    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createresolvingfunctions
    fn create_resolving_functions(
        promise: &JsObject,
        context: &mut Context,
    ) -> JsResult<ResolvingFunctionsRecord> {
        // TODO: can this not be a rust struct?
        // 1. Let alreadyResolved be the Record { [[Value]]: false }.
        let already_resolved = JsObject::empty();
        already_resolved.set("Value", JsValue::from(false), true, context)?;

        let resolve_captures = RejectResolveCaptures {
            already_resolved: already_resolved.clone(),
            promise: promise.clone(),
        };

        // 2. Let stepsResolve be the algorithm steps defined in Promise Resolve Functions.
        // 3. Let lengthResolve be the number of non-optional parameters of the function definition in Promise Resolve Functions.
        // 4. Let resolve be CreateBuiltinFunction(stepsResolve, lengthResolve, "", « [[Promise]], [[AlreadyResolved]] »).
        let resolve = FunctionBuilder::closure_with_captures(
            context,
            |this, args, captures, context| {
                // https://tc39.es/ecma262/#sec-promise-resolve-functions

                // 1. Let F be the active function object.
                // 2. Assert: F has a [[Promise]] internal slot whose value is an Object.
                // 3. Let promise be F.[[Promise]].
                // 4. Let alreadyResolved be F.[[AlreadyResolved]].
                let RejectResolveCaptures {
                    promise,
                    already_resolved,
                } = captures;

                // 5. If alreadyResolved.[[Value]] is true, return undefined.
                if already_resolved
                    .get("Value", context)?
                    .as_boolean()
                    .unwrap_or(false)
                {
                    return Ok(JsValue::Undefined);
                }

                // 6. Set alreadyResolved.[[Value]] to true.
                already_resolved.set("Value", true, true, context)?;

                let resolution = args.get_or_undefined(0);

                // 7. If SameValue(resolution, promise) is true, then
                if JsValue::same_value(resolution, &promise.clone().into()) {
                    //   a. Let selfResolutionError be a newly created TypeError object.
                    let self_resolution_error =
                        context.construct_type_error("SameValue(resolution, promise) is true");

                    //   b. Perform RejectPromise(promise, selfResolutionError).
                    promise
                        .borrow_mut()
                        .as_promise_mut()
                        .expect("Expected promise to be a Promise")
                        .reject(&self_resolution_error, context)?;

                    //   c. Return undefined.
                    return Ok(JsValue::Undefined);
                }

                // 8. If Type(resolution) is not Object, then
                if !resolution.is_object() {
                    // a. Perform FulfillPromise(promise, resolution).
                    promise
                        .borrow_mut()
                        .as_promise_mut()
                        .expect("Expected promise to be a Promise")
                        .fulfill(resolution, context)?;

                    //   b. Return undefined.
                    return Ok(JsValue::Undefined);
                }

                // 9. Let then be Completion(Get(resolution, "then")).
                let then = resolution
                    .as_object()
                    .unwrap_or_else(|| unreachable!())
                    .get("then", context);

                let then = match then {
                    // 10. If then is an abrupt completion, then
                    Err(value) => {
                        //   a. Perform RejectPromise(promise, then.[[Value]]).
                        promise
                            .borrow_mut()
                            .as_promise_mut()
                            .expect("Expected promise to be a Promise")
                            .reject(&value, context)?;

                        //   b. Return undefined.
                        return Ok(JsValue::Undefined);
                    }
                    Ok(then) => then,
                };

                // 11. Let thenAction be then.[[Value]].
                let then_action = then
                    .as_object()
                    .expect("rsolution.[[then]] should be an object")
                    .get("Value", context)?;

                // 12. If IsCallable(thenAction) is false, then
                if !then_action.is_callable() {
                    // a. Perform FulfillPromise(promise, resolution).
                    promise
                        .borrow_mut()
                        .as_promise_mut()
                        .expect("Expected promise to be a Promise")
                        .fulfill(resolution, context)?;

                    //   b. Return undefined.
                    return Ok(JsValue::Undefined);
                }

                // 13. Let thenJobCallback be HostMakeJobCallback(thenAction).
                let then_job_callback = JobCallback::make_job_callback(then_action);

                // 14. Let job be NewPromiseResolveThenableJob(promise, resolution, thenJobCallback).
                let job: JobCallback = PromiseJob::new_promise_resolve_thenable_job(
                    promise.clone(),
                    resolution.clone(),
                    then_job_callback,
                    context,
                );

                // 15. Perform HostEnqueuePromiseJob(job.[[Job]], job.[[Realm]]).
                context.host_enqueue_promise_job(Box::new(job));

                // 16. Return undefined.
                Ok(JsValue::Undefined)
            },
            resolve_captures,
        )
        .name("")
        .length(1)
        .constructor(false)
        .build();

        // 5. Set resolve.[[Promise]] to promise.
        resolve.set("Promise", promise.clone(), true, context)?;

        // 6. Set resolve.[[AlreadyResolved]] to alreadyResolved.
        resolve.set("AlreadyResolved", already_resolved.clone(), true, context)?;

        let reject_captures = RejectResolveCaptures {
            promise: promise.clone(),
            already_resolved: already_resolved.clone(),
        };

        // 7. Let stepsReject be the algorithm steps defined in Promise Reject Functions.
        // 8. Let lengthReject be the number of non-optional parameters of the function definition in Promise Reject Functions.
        // 9. Let reject be CreateBuiltinFunction(stepsReject, lengthReject, "", « [[Promise]], [[AlreadyResolved]] »).
        let reject = FunctionBuilder::closure_with_captures(
            context,
            |this, args, captures, context| {
                // https://tc39.es/ecma262/#sec-promise-reject-functions

                // 1. Let F be the active function object.
                // 2. Assert: F has a [[Promise]] internal slot whose value is an Object.
                // 3. Let promise be F.[[Promise]].
                // 4. Let alreadyResolved be F.[[AlreadyResolved]].
                let RejectResolveCaptures {
                    promise,
                    already_resolved,
                } = captures;

                // 5. If alreadyResolved.[[Value]] is true, return undefined.
                if already_resolved
                    .get("Value", context)?
                    .as_boolean()
                    .unwrap_or(false)
                {
                    return Ok(JsValue::Undefined);
                }

                // 6. Set alreadyResolved.[[Value]] to true.
                already_resolved.set("Value", true, true, context)?;

                let reason = args.get_or_undefined(0);
                // 7. Perform RejectPromise(promise, reason).
                promise
                    .borrow_mut()
                    .as_promise_mut()
                    .expect("Expected promise to be a Promise")
                    .reject(reason, context)?;

                // 8. Return undefined.
                Ok(JsValue::Undefined)
            },
            reject_captures,
        )
        .name("")
        .length(1)
        .constructor(false)
        .build();

        // 10. Set reject.[[Promise]] to promise.
        reject.set("Promise", promise.clone(), true, context)?;

        // 11. Set reject.[[AlreadyResolved]] to alreadyResolved.
        reject.set("AlreadyResolved", already_resolved, true, context)?;

        // 12. Return the Record { [[Resolve]]: resolve, [[Reject]]: reject }.
        let resolve = resolve.conv::<JsValue>();
        let reject = reject.conv::<JsValue>();
        Ok(ResolvingFunctionsRecord { resolve, reject })
    }

    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-fulfillpromise
    pub fn fulfill(&mut self, value: &JsValue, context: &mut Context) -> JsResult<()> {
        // 1. Assert: The value of promise.[[PromiseState]] is pending.
        match self.promise_state {
            PromiseState::Pending => (),
            _ => return context.throw_error("Expected promise.[[PromiseState]] to be pending"),
        }

        // 2. Let reactions be promise.[[PromiseFulfillReactions]].
        let reactions = &self.promise_fulfill_reactions;

        // 7. Perform TriggerPromiseReactions(reactions, value).
        Self::trigger_promise_reactions(reactions, value, context);
        // reordering this statement does not affect the semantics

        // 3. Set promise.[[PromiseResult]] to value.
        self.promise_result = Some(value.clone());

        // 4. Set promise.[[PromiseFulfillReactions]] to undefined.
        self.promise_fulfill_reactions = vec![];

        // 5. Set promise.[[PromiseRejectReactions]] to undefined.
        self.promise_reject_reactions = vec![];

        // 6. Set promise.[[PromiseState]] to fulfilled.
        self.promise_state = PromiseState::Fulfilled;

        // 8. Return unused.
        Ok(())
    }

    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-rejectpromise
    pub fn reject(&mut self, reason: &JsValue, context: &mut Context) -> JsResult<()> {
        // 1. Assert: The value of promise.[[PromiseState]] is pending.
        match self.promise_state {
            PromiseState::Pending => (),
            _ => return context.throw_error("Expected promise.[[PromiseState]] to be pending"),
        }

        // 2. Let reactions be promise.[[PromiseRejectReactions]].
        let reactions = &self.promise_reject_reactions;

        // 8. Perform TriggerPromiseReactions(reactions, reason).
        Self::trigger_promise_reactions(reactions, reason, context);
        // reordering this statement does not affect the semantics

        // 3. Set promise.[[PromiseResult]] to reason.
        self.promise_result = Some(reason.clone());

        // 4. Set promise.[[PromiseFulfillReactions]] to undefined.
        self.promise_fulfill_reactions = vec![];

        // 5. Set promise.[[PromiseRejectReactions]] to undefined.
        self.promise_reject_reactions = vec![];

        // 6. Set promise.[[PromiseState]] to rejected.
        self.promise_state = PromiseState::Rejected;

        // 7. If promise.[[PromiseIsHandled]] is false, perform HostPromiseRejectionTracker(promise, "reject").
        if !self.promise_is_handled {
            // TODO
        }

        // 9. Return unused.
        Ok(())
    }

    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-triggerpromisereactions
    pub fn trigger_promise_reactions(
        reactions: &[ReactionRecord],
        argument: &JsValue,
        context: &mut Context,
    ) {
        // 1. For each element reaction of reactions, do
        for reaction in reactions {
            // a. Let job be NewPromiseReactionJob(reaction, argument).
            let job =
                PromiseJob::new_promise_reaction_job(reaction.clone(), argument.clone(), context);

            // b. Perform HostEnqueuePromiseJob(job.[[Job]], job.[[Realm]]).
            context.host_enqueue_promise_job(Box::new(job));
        }

        // 2. Return unused.
    }

    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.prototype.then
    pub fn then(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let promise be the this value.
        let promise = this;

        // 2. If IsPromise(promise) is false, throw a TypeError exception.
        let promise_obj = match promise.as_object() {
            Some(obj) => obj,
            None => return context.throw_type_error("IsPromise(promise) is false"),
        };

        // 3. Let C be ? SpeciesConstructor(promise, %Promise%).
        let c = promise_obj.species_constructor(StandardConstructors::promise, context)?;

        // 4. Let resultCapability be ? NewPromiseCapability(C).
        let result_capability = PromiseCapability::new(&c.into(), context)?;

        let on_fulfilled = args.get_or_undefined(0).clone();
        let on_rejected = args.get_or_undefined(1).clone();

        // 5. Return PerformPromiseThen(promise, onFulfilled, onRejected, resultCapability).
        promise_obj
            .borrow_mut()
            .as_promise_mut()
            .expect("IsPromise(promise) is false")
            .perform_promise_then(on_fulfilled, on_rejected, Some(result_capability), context)
            .pipe(Ok)
    }

    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromisethen
    fn perform_promise_then(
        &mut self,
        on_fulfilled: JsValue,
        on_rejected: JsValue,
        result_capability: Option<PromiseCapability>,
        context: &mut Context,
    ) -> JsValue {
        // 1. Assert: IsPromise(promise) is true.

        // 2. If resultCapability is not present, then
        //   a. Set resultCapability to undefined.

        let on_fulfilled_job_callback: Option<JobCallback> =
        // 3. If IsCallable(onFulfilled) is false, then
            if on_fulfilled.is_callable() {
                // 4. Else,
                //   a. Let onFulfilledJobCallback be HostMakeJobCallback(onFulfilled).
                Some(JobCallback::make_job_callback(on_fulfilled))
            } else {
                //   a. Let onFulfilledJobCallback be empty.
                None
            };

        let on_rejected_job_callback: Option<JobCallback> =
        // 5. If IsCallable(onRejected) is false, then
            if on_rejected.is_callable() {
                // 6. Else,
                //   a. Let onRejectedJobCallback be HostMakeJobCallback(onRejected).
                Some(JobCallback::make_job_callback(on_rejected))
            } else {
                //   a. Let onRejectedJobCallback be empty.
                None
            };

        // 7. Let fulfillReaction be the PromiseReaction { [[Capability]]: resultCapability, [[Type]]: Fulfill, [[Handler]]: onFulfilledJobCallback }.
        let fulfill_reaction = ReactionRecord {
            promise_capability: result_capability.clone(),
            reaction_type: ReactionType::Fulfill,
            handler: on_fulfilled_job_callback,
        };

        // 8. Let rejectReaction be the PromiseReaction { [[Capability]]: resultCapability, [[Type]]: Reject, [[Handler]]: onRejectedJobCallback }.
        let reject_reaction = ReactionRecord {
            promise_capability: result_capability.clone(),
            reaction_type: ReactionType::Reject,
            handler: on_rejected_job_callback,
        };

        match self.promise_state {
            // 9. If promise.[[PromiseState]] is pending, then
            PromiseState::Pending => {
                //   a. Append fulfillReaction as the last element of the List that is promise.[[PromiseFulfillReactions]].
                self.promise_fulfill_reactions.push(fulfill_reaction);

                //   b. Append rejectReaction as the last element of the List that is promise.[[PromiseRejectReactions]].
                self.promise_reject_reactions.push(reject_reaction);
            }

            // 10. Else if promise.[[PromiseState]] is fulfilled, then
            PromiseState::Fulfilled => {
                //   a. Let value be promise.[[PromiseResult]].
                let value = self
                    .promise_result
                    .clone()
                    .expect("promise.[[PromiseResult]] cannot be empty");

                //   b. Let fulfillJob be NewPromiseReactionJob(fulfillReaction, value).
                let fulfill_job =
                    PromiseJob::new_promise_reaction_job(fulfill_reaction, value, context);

                //   c. Perform HostEnqueuePromiseJob(fulfillJob.[[Job]], fulfillJob.[[Realm]]).
                context.host_enqueue_promise_job(Box::new(fulfill_job));
            }

            // 11. Else,
            //   a. Assert: The value of promise.[[PromiseState]] is rejected.
            PromiseState::Rejected => {
                //   b. Let reason be promise.[[PromiseResult]].
                let reason = self
                    .promise_result
                    .clone()
                    .expect("promise.[[PromiseResult]] cannot be empty");

                //   c. If promise.[[PromiseIsHandled]] is false, perform HostPromiseRejectionTracker(promise, "handle").
                if !self.promise_is_handled {
                    // HostPromiseRejectionTracker(promise, "handle")
                    todo!(); // TODO
                }

                //   d. Let rejectJob be NewPromiseReactionJob(rejectReaction, reason).
                let reject_job =
                    PromiseJob::new_promise_reaction_job(reject_reaction, reason, context);

                //   e. Perform HostEnqueuePromiseJob(rejectJob.[[Job]], rejectJob.[[Realm]]).
                context.host_enqueue_promise_job(Box::new(reject_job));

                // 12. Set promise.[[PromiseIsHandled]] to true.
                self.promise_is_handled = true;
            }
        }

        match result_capability {
            // 13. If resultCapability is undefined, then
            //   a. Return undefined.
            None => JsValue::Undefined,

            // 14. Else,
            //   a. Return resultCapability.[[Promise]].
            Some(result_capability) => result_capability.promise.clone(),
        }
    }
}
