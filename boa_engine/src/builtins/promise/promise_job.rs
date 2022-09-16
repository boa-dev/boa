use super::{Promise, PromiseCapability};
use crate::{
    builtins::promise::{ReactionRecord, ReactionType},
    job::JobCallback,
    object::{FunctionBuilder, JsObject},
    Context, JsValue,
};
use boa_gc::{Finalize, Trace};

#[derive(Debug, Clone, Copy)]
pub(crate) struct PromiseJob;

impl PromiseJob {
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-newpromisereactionjob
    pub(crate) fn new_promise_reaction_job(
        reaction: ReactionRecord,
        argument: JsValue,
        context: &mut Context,
    ) -> JobCallback {
        #[derive(Debug, Trace, Finalize)]
        struct ReactionJobCaptures {
            reaction: ReactionRecord,
            argument: JsValue,
        }

        // 1. Let job be a new Job Abstract Closure with no parameters that captures reaction and argument and performs the following steps when called:
        let job = FunctionBuilder::closure_with_captures(
            context,
            |_this, _args, captures, context| {
                let ReactionJobCaptures { reaction, argument } = captures;

                let ReactionRecord {
                    //   a. Let promiseCapability be reaction.[[Capability]].
                    promise_capability,
                    //   b. Let type be reaction.[[Type]].
                    reaction_type,
                    //   c. Let handler be reaction.[[Handler]].
                    handler,
                } = reaction;

                let handler_result = match handler {
                    // d. If handler is empty, then
                    None => match reaction_type {
                        // i. If type is Fulfill, let handlerResult be NormalCompletion(argument).
                        ReactionType::Fulfill => Ok(argument.clone()),
                        // ii. Else,
                        //   1. Assert: type is Reject.
                        ReactionType::Reject => {
                            // 2. Let handlerResult be ThrowCompletion(argument).
                            Err(argument.clone())
                        }
                    },
                    //   e. Else, let handlerResult be Completion(HostCallJobCallback(handler, undefined, « argument »)).
                    Some(handler) => handler
                        .call_job_callback(&JsValue::Undefined, &[argument.clone()], context)
                        .map_err(|e| e.to_opaque(context)),
                };

                match promise_capability {
                    None => {
                        // f. If promiseCapability is undefined, then
                        //    i. Assert: handlerResult is not an abrupt completion.
                        assert!(
                            handler_result.is_ok(),
                            "Assertion: <handlerResult is not an abrupt completion> failed"
                        );

                        // ii. Return empty.
                        Ok(JsValue::Undefined)
                    }
                    Some(promise_capability_record) => {
                        // g. Assert: promiseCapability is a PromiseCapability Record.
                        let PromiseCapability {
                            promise: _,
                            resolve,
                            reject,
                        } = promise_capability_record;

                        match handler_result {
                            // h. If handlerResult is an abrupt completion, then
                            Err(value) => {
                                // i. Return ? Call(promiseCapability.[[Reject]], undefined, « handlerResult.[[Value]] »).
                                context.call(&reject.clone().into(), &JsValue::Undefined, &[value])
                            }

                            // i. Else,
                            Ok(value) => {
                                // i. Return ? Call(promiseCapability.[[Resolve]], undefined, « handlerResult.[[Value]] »).
                                context.call(&resolve.clone().into(), &JsValue::Undefined, &[value])
                            }
                        }
                    }
                }
            },
            ReactionJobCaptures { reaction, argument },
        )
        .build()
        .into();

        // 2. Let handlerRealm be null.
        // 3. If reaction.[[Handler]] is not empty, then
        //   a. Let getHandlerRealmResult be Completion(GetFunctionRealm(reaction.[[Handler]].[[Callback]])).
        //   b. If getHandlerRealmResult is a normal completion, set handlerRealm to getHandlerRealmResult.[[Value]].
        //   c. Else, set handlerRealm to the current Realm Record.
        //   d. NOTE: handlerRealm is never null unless the handler is undefined. When the handler is a revoked Proxy and no ECMAScript code runs, handlerRealm is used to create error objects.
        // 4. Return the Record { [[Job]]: job, [[Realm]]: handlerRealm }.
        JobCallback::make_job_callback(job)
    }

    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-newpromiseresolvethenablejob
    pub(crate) fn new_promise_resolve_thenable_job(
        promise_to_resolve: JsObject,
        thenable: JsValue,
        then: JobCallback,
        context: &mut Context,
    ) -> JobCallback {
        // 1. Let job be a new Job Abstract Closure with no parameters that captures promiseToResolve, thenable, and then and performs the following steps when called:
        let job = FunctionBuilder::closure_with_captures(
            context,
            |_this: &JsValue, _args: &[JsValue], captures, context: &mut Context| {
                let JobCapture {
                    promise_to_resolve,
                    thenable,
                    then,
                } = captures;

                //    a. Let resolvingFunctions be CreateResolvingFunctions(promiseToResolve).
                let resolving_functions =
                    Promise::create_resolving_functions(promise_to_resolve, context);

                //    b. Let thenCallResult be Completion(HostCallJobCallback(then, thenable, « resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]] »)).
                let then_call_result = then.call_job_callback(
                    thenable,
                    &[
                        resolving_functions.resolve,
                        resolving_functions.reject.clone(),
                    ],
                    context,
                );

                //    c. If thenCallResult is an abrupt completion, then
                if let Err(value) = then_call_result {
                    let value = value.to_opaque(context);
                    //    i. Return ? Call(resolvingFunctions.[[Reject]], undefined, « thenCallResult.[[Value]] »).
                    return context.call(
                        &resolving_functions.reject,
                        &JsValue::Undefined,
                        &[value],
                    );
                }

                //    d. Return ? thenCallResult.
                then_call_result
            },
            JobCapture::new(promise_to_resolve, thenable, then),
        )
        .build();

        // 2. Let getThenRealmResult be Completion(GetFunctionRealm(then.[[Callback]])).
        // 3. If getThenRealmResult is a normal completion, let thenRealm be getThenRealmResult.[[Value]].
        // 4. Else, let thenRealm be the current Realm Record.
        // 5. NOTE: thenRealm is never null. When then.[[Callback]] is a revoked Proxy and no code runs, thenRealm is used to create error objects.
        // 6. Return the Record { [[Job]]: job, [[Realm]]: thenRealm }.
        JobCallback::make_job_callback(job.into())
    }
}

#[derive(Debug, Trace, Finalize)]
struct JobCapture {
    promise_to_resolve: JsObject,
    thenable: JsValue,
    then: JobCallback,
}

impl JobCapture {
    fn new(promise_to_resolve: JsObject, thenable: JsValue, then: JobCallback) -> Self {
        Self {
            promise_to_resolve,
            thenable,
            then,
        }
    }
}
