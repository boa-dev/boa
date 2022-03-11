//! This module implements the global `Promise` object.

#![allow(dead_code, unused_results, unused_variables)]

#[cfg(test)]
mod tests;

pub mod fetch;
mod promise_job;

use boa_gc::{Finalize, Trace};
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

impl ResolvedRecord {
    fn new(value: bool) -> Self {
        Self { value }
    }
}

#[derive(Debug, Trace, Finalize)]
struct RejectResolveCaptures {
    promise: JsObject,
    already_resolved: JsObject,
}

impl RejectResolveCaptures {
    fn new(promise: JsObject, already_resolved: JsObject) -> Self {
        Self {
            promise,
            already_resolved,
        }
    }
}

impl Promise {
    const LENGTH: usize = 1;

    /// https://tc39.es/ecma262/#sec-promise-executor
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
            let _ = context.call(&resolving_functions.reject, &JsValue::Undefined, &[value]);
        }

        // 11. Return promise.
        promise.conv::<JsValue>().pipe(Ok)
    }

    /// https://tc39.es/ecma262/#sec-createresolvingfunctions
    fn create_resolving_functions(
        promise: &JsObject,
        context: &mut Context,
    ) -> JsResult<ResolvingFunctionsRecord> {
        // FIXME: can this not be a rust struct?
        // 1. Let alreadyResolved be the Record { [[Value]]: false }.
        let already_resolved = JsObject::empty();
        already_resolved.set("Value", JsValue::from(false), true, context)?;

        let resolve_captures =
            RejectResolveCaptures::new(promise.clone(), already_resolved.clone());

        // 2. Let stepsResolve be the algorithm steps defined in Promise Resolve Functions.
        // 3. Let lengthResolve be the number of non-optional parameters of the function definition in Promise Resolve Functions.
        // 4. Let resolve be CreateBuiltinFunction(stepsResolve, lengthResolve, "", « [[Promise]], [[AlreadyResolved]] »).
        let resolve = FunctionBuilder::closure_with_captures(
            context,
            |this, args, captures, context| {
                // https://tc39.es/ecma262/#sec-promise-resolve-functions
                let resolution = args.get_or_undefined(0);

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
                    .get("Value", context)
                    .expect("msg")
                    .as_boolean()
                    .expect("msg")
                {
                    return Ok(JsValue::Undefined);
                }

                // 6. Set alreadyResolved.[[Value]] to true.
                already_resolved.set("Value", true, true, context)?;

                // TODO
                // 7. If SameValue(resolution, promise) is true, then
                //   a. Let selfResolutionError be a newly created TypeError object.
                //   b. Perform RejectPromise(promise, selfResolutionError).
                //   c. Return undefined.

                // 8. If Type(resolution) is not Object, then
                if !resolution.is_object() {
                    // a. Perform FulfillPromise(promise, resolution).
                    promise
                        .borrow_mut()
                        .as_promise_mut()
                        .expect("msg")
                        .fulfill(resolution, context);

                    //   b. Return undefined.
                    return Ok(JsValue::Undefined);
                }

                // 9. Let then be Completion(Get(resolution, "then")).
                let then = resolution
                    .as_object()
                    .expect("msg")
                    .get("then", context)
                    .expect("msg");

                // TODO
                // 10. If then is an abrupt completion, then
                // if let Err(value) = then {
                //   a. Perform RejectPromise(promise, then.[[Value]]).

                //   b. Return undefined.
                // return Ok(JsValue::Undefined)
                // }

                // 11. Let thenAction be then.[[Value]].
                let then_action = then;

                // 12. If IsCallable(thenAction) is false, then
                if !then_action.is_callable() {
                    //   a. Perform FulfillPromise(promise, resolution).
                    promise
                        .borrow_mut()
                        .as_promise_mut()
                        .expect("msg")
                        .fulfill(resolution, context);

                    //   b. Return undefined.
                    return Ok(JsValue::Undefined);
                }

                // 13. Let thenJobCallback be HostMakeJobCallback(thenAction).
                let then_job_callback = JobCallback::make_job_callback(then_action);

                // 14. Let job be NewPromiseResolveThenableJob(promise, resolution, thenJobCallback).
                let job: JobCallback = PromiseJob::new_promise_resolve_thenable_job(
                    promise.clone(),
                    resolution.clone().into(),
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

        let reject_captures = RejectResolveCaptures::new(promise.clone(), already_resolved.clone());

        // 7. Let stepsReject be the algorithm steps defined in Promise Reject Functions.
        // 8. Let lengthReject be the number of non-optional parameters of the function definition in Promise Reject Functions.
        // 9. Let reject be CreateBuiltinFunction(stepsReject, lengthReject, "", « [[Promise]], [[AlreadyResolved]] »).
        let reject = FunctionBuilder::closure_with_captures(
            context,
            |this, args, captures, context| {
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
                    .get("Value", context)
                    .expect("msg")
                    .as_boolean()
                    .expect("msg")
                {
                    return Ok(JsValue::Undefined);
                }

                // 6. Set alreadyResolved.[[Value]] to true.
                already_resolved.set("Value", true, true, context)?;

                // 7. Perform RejectPromise(promise, reason).
                // TODO

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

    /// https://tc39.es/ecma262/#sec-fulfillpromise
    pub fn fulfill(&mut self, value: &JsValue, context: &mut Context) -> () {
        // TODO: check if statement change of 7. to 2. changes the semantics also
        // 1. Assert: The value of promise.[[PromiseState]] is pending.
        match self.promise_state {
            PromiseState::Pending => (),
            _ => (), // TODO: throw assertion error
        }

        // 2. Let reactions be promise.[[PromiseFulfillReactions]].
        let reactions = &self.promise_fulfill_reactions;

        // 7. Perform TriggerPromiseReactions(reactions, value).
        Promise::trigger_promise_reactions(reactions, value.clone(), context);

        // 3. Set promise.[[PromiseResult]] to value.
        self.promise_result = Some(value.clone());

        // 4. Set promise.[[PromiseFulfillReactions]] to undefined.
        self.promise_fulfill_reactions = vec![];

        // 5. Set promise.[[PromiseRejectReactions]] to undefined.
        self.promise_reject_reactions = vec![];

        // 6. Set promise.[[PromiseState]] to fulfilled.
        self.promise_state = PromiseState::Fulfilled;

        // 8. Return unused.
        return ();
    }

    /// https://tc39.es/ecma262/#sec-triggerpromisereactions
    pub fn trigger_promise_reactions(
        reactions: &Vec<ReactionRecord>,
        argument: JsValue,
        context: &mut Context,
    ) {
        // 1. For each element reaction of reactions, do
        for reaction in reactions {
            // a. Let job be NewPromiseReactionJob(reaction, argument).
            let job = PromiseJob::new_promise_reaction_job(reaction.clone(), argument.clone(), context);

            // b. Perform HostEnqueuePromiseJob(job.[[Job]], job.[[Realm]]).
            context.host_enqueue_promise_job(Box::new(job))
        }

        // 2. Return unused.
        ()
    }
}
