//! This module implements the global `Promise` object.

#[cfg(test)]
mod tests;

mod promise_job;

use self::promise_job::PromiseJob;
use super::{iterable::IteratorRecord, JsArgs};
use crate::{
    builtins::{error::ErrorKind, Array, BuiltIn},
    context::intrinsics::StandardConstructors,
    error::JsNativeError,
    job::JobCallback,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        JsFunction, JsObject, ObjectData,
    },
    property::{Attribute, PropertyDescriptorBuilder},
    symbol::WellKnownSymbols,
    value::JsValue,
    Context, JsError, JsResult,
};
use boa_gc::{Finalize, Gc, GcCell, Trace};
use boa_profiler::Profiler;
use std::{cell::Cell, rc::Rc};
use tap::{Conv, Pipe};

/// `IfAbruptRejectPromise ( value, capability )`
///
/// `IfAbruptRejectPromise` is a shorthand for a sequence of algorithm steps that use a `PromiseCapability` Record.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ifabruptrejectpromise
macro_rules! if_abrupt_reject_promise {
    ($value:ident, $capability:expr, $context: expr) => {
        let $value = match $value {
            // 1. If value is an abrupt completion, then
            Err(err) => {
                let err = err.to_opaque($context);
                // a. Perform ? Call(capability.[[Reject]], undefined, « value.[[Value]] »).
                $context.call(
                    &$capability.reject().clone().into(),
                    &JsValue::undefined(),
                    &[err],
                )?;

                // b. Return capability.[[Promise]].
                return Ok($capability.promise().clone().into());
            }
            // 2. Else if value is a Completion Record, set value to value.[[Value]].
            Ok(value) => value,
        };
    };
}

pub(crate) use if_abrupt_reject_promise;

#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) enum PromiseState {
    Pending,
    Fulfilled(JsValue),
    Rejected(JsValue),
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Promise {
    promise_state: PromiseState,
    promise_fulfill_reactions: Vec<ReactionRecord>,
    promise_reject_reactions: Vec<ReactionRecord>,
    promise_is_handled: bool,
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct ReactionRecord {
    promise_capability: Option<PromiseCapability>,
    #[unsafe_ignore_trace]
    reaction_type: ReactionType,
    handler: Option<JobCallback>,
}

#[derive(Debug, Clone, Copy)]
enum ReactionType {
    Fulfill,
    Reject,
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct PromiseCapability {
    promise: JsObject,
    resolve: JsFunction,
    reject: JsFunction,
}

impl PromiseCapability {
    /// `NewPromiseCapability ( C )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-newpromisecapability
    pub(crate) fn new(c: &JsValue, context: &mut Context) -> JsResult<Self> {
        #[derive(Debug, Clone, Trace, Finalize)]
        struct RejectResolve {
            reject: JsValue,
            resolve: JsValue,
        }

        match c.as_constructor() {
            // 1. If IsConstructor(C) is false, throw a TypeError exception.
            None => Err(JsNativeError::typ()
                .with_message("PromiseCapability: expected constructor")
                .into()),
            Some(c) => {
                let c = c.clone();

                // 2. NOTE: C is assumed to be a constructor function that supports the parameter conventions of the Promise constructor (see 27.2.3.1).
                // 3. Let promiseCapability be the PromiseCapability Record { [[Promise]]: undefined, [[Resolve]]: undefined, [[Reject]]: undefined }.
                let promise_capability = Gc::new(GcCell::new(RejectResolve {
                    reject: JsValue::undefined(),
                    resolve: JsValue::undefined(),
                }));

                // 4. Let executorClosure be a new Abstract Closure with parameters (resolve, reject) that captures promiseCapability and performs the following steps when called:
                // 5. Let executor be CreateBuiltinFunction(executorClosure, 2, "", « »).
                let executor = FunctionBuilder::closure_with_captures(
                    context,
                    |_this, args: &[JsValue], captures, _| {
                        let mut promise_capability = captures.borrow_mut();
                        // a. If promiseCapability.[[Resolve]] is not undefined, throw a TypeError exception.
                        if !promise_capability.resolve.is_undefined() {
                            return Err(JsNativeError::typ()
                                .with_message("promiseCapability.[[Resolve]] is not undefined")
                                .into());
                        }

                        // b. If promiseCapability.[[Reject]] is not undefined, throw a TypeError exception.
                        if !promise_capability.reject.is_undefined() {
                            return Err(JsNativeError::typ()
                                .with_message("promiseCapability.[[Reject]] is not undefined")
                                .into());
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
                    promise_capability.clone(),
                )
                .name("")
                .length(2)
                .build()
                .into();

                // 6. Let promise be ? Construct(C, « executor »).
                let promise = c.construct(&[executor], Some(&c), context)?;

                let promise_capability = promise_capability.borrow();

                let resolve = promise_capability.resolve.clone();
                let reject = promise_capability.reject.clone();

                // 7. If IsCallable(promiseCapability.[[Resolve]]) is false, throw a TypeError exception.
                let resolve = resolve
                    .as_object()
                    .cloned()
                    .and_then(JsFunction::from_object)
                    .ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("promiseCapability.[[Resolve]] is not callable")
                    })?;

                // 8. If IsCallable(promiseCapability.[[Reject]]) is false, throw a TypeError exception.
                let reject = reject
                    .as_object()
                    .cloned()
                    .and_then(JsFunction::from_object)
                    .ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("promiseCapability.[[Reject]] is not callable")
                    })?;

                // 9. Set promiseCapability.[[Promise]] to promise.
                // 10. Return promiseCapability.
                Ok(PromiseCapability {
                    promise,
                    resolve,
                    reject,
                })
            }
        }
    }

    /// Returns the promise object.
    pub(crate) fn promise(&self) -> &JsObject {
        &self.promise
    }

    /// Returns the resolve function.
    pub(crate) fn resolve(&self) -> &JsFunction {
        &self.resolve
    }

    /// Returns the reject function.
    pub(crate) fn reject(&self) -> &JsFunction {
        &self.reject
    }
}

impl BuiltIn for Promise {
    const NAME: &'static str = "Promise";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let get_species = FunctionBuilder::native(context, Self::get_species)
            .name("get [Symbol.species]")
            .constructor(false)
            .build();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().promise().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .static_method(Self::all, "all", 1)
        .static_method(Self::all_settled, "allSettled", 1)
        .static_method(Self::any, "any", 1)
        .static_method(Self::race, "race", 1)
        .static_method(Self::reject, "reject", 1)
        .static_method(Self::resolve, "resolve", 1)
        .static_accessor(
            WellKnownSymbols::species(),
            Some(get_species),
            None,
            Attribute::CONFIGURABLE,
        )
        .method(Self::then, "then", 2)
        .method(Self::catch, "catch", 1)
        .method(Self::finally, "finally", 1)
        // <https://tc39.es/ecma262/#sec-promise.prototype-@@tostringtag>
        .property(
            WellKnownSymbols::to_string_tag(),
            Self::NAME,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

#[derive(Debug)]
struct ResolvingFunctionsRecord {
    resolve: JsValue,
    reject: JsValue,
}

impl Promise {
    const LENGTH: usize = 1;

    /// Gets the current state of the promise.
    pub(crate) fn state(&self) -> &PromiseState {
        &self.promise_state
    }

    /// `Promise ( executor )`
    ///
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
            return Err(JsNativeError::typ()
                .with_message("Promise NewTarget cannot be undefined")
                .into());
        }

        let executor = args.get_or_undefined(0);

        // 2. If IsCallable(executor) is false, throw a TypeError exception.
        if !executor.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("Promise executor is not callable")
                .into());
        }

        // 3. Let promise be ? OrdinaryCreateFromConstructor(NewTarget, "%Promise.prototype%", « [[PromiseState]], [[PromiseResult]], [[PromiseFulfillReactions]], [[PromiseRejectReactions]], [[PromiseIsHandled]] »).
        let promise =
            get_prototype_from_constructor(new_target, StandardConstructors::promise, context)?;

        let promise = JsObject::from_proto_and_data(
            promise,
            ObjectData::promise(Self {
                // 4. Set promise.[[PromiseState]] to pending.
                promise_state: PromiseState::Pending,
                // 5. Set promise.[[PromiseFulfillReactions]] to a new empty List.
                promise_fulfill_reactions: Vec::new(),
                // 6. Set promise.[[PromiseRejectReactions]] to a new empty List.
                promise_reject_reactions: Vec::new(),
                // 7. Set promise.[[PromiseIsHandled]] to false.
                promise_is_handled: false,
            }),
        );

        // 8. Let resolvingFunctions be CreateResolvingFunctions(promise).
        let resolving_functions = Self::create_resolving_functions(&promise, context);

        // 9. Let completion Completion(Call(executor, undefined, « resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]] »)be ).
        let completion = context.call(
            executor,
            &JsValue::Undefined,
            &[
                resolving_functions.resolve,
                resolving_functions.reject.clone(),
            ],
        );

        // 10. If completion is an abrupt completion, then
        if let Err(e) = completion {
            let e = e.to_opaque(context);
            // a. Perform ? Call(resolvingFunctions.[[Reject]], undefined, « completion.[[Value]] »).
            context.call(&resolving_functions.reject, &JsValue::Undefined, &[e])?;
        }

        // 11. Return promise.
        promise.conv::<JsValue>().pipe(Ok)
    }

    /// `Promise.all ( iterable )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.all
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/all
    pub(crate) fn all(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let C be the this value.
        let c = this;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(c, context)?;

        // Note: We already checked that `C` is a constructor in `NewPromiseCapability(C)`.
        let c = c.as_object().expect("must be a constructor");

        // 3. Let promiseResolve be Completion(GetPromiseResolve(C)).
        let promise_resolve = Self::get_promise_resolve(c, context);

        // 4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
        if_abrupt_reject_promise!(promise_resolve, promise_capability, context);

        // 5. Let iteratorRecord be Completion(GetIterator(iterable)).
        let iterator_record = args.get_or_undefined(0).get_iterator(context, None, None);

        // 6. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
        if_abrupt_reject_promise!(iterator_record, promise_capability, context);
        let mut iterator_record = iterator_record;

        // 7. Let result be Completion(PerformPromiseAll(iteratorRecord, C, promiseCapability, promiseResolve)).
        let mut result = Self::perform_promise_all(
            &mut iterator_record,
            c,
            &promise_capability,
            &promise_resolve,
            context,
        )
        .map(JsValue::from);

        // 8. If result is an abrupt completion, then
        if result.is_err() {
            // a. If iteratorRecord.[[Done]] is false, set result to Completion(IteratorClose(iteratorRecord, result)).
            if !iterator_record.done() {
                result = iterator_record.close(result, context);
            }

            // b. IfAbruptRejectPromise(result, promiseCapability).
            if_abrupt_reject_promise!(result, promise_capability, context);

            return Ok(result);
        }

        // 9. Return ? result.
        result
    }

    /// `PerformPromiseAll ( iteratorRecord, constructor, resultCapability, promiseResolve )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromiseall
    fn perform_promise_all(
        iterator_record: &mut IteratorRecord,
        constructor: &JsObject,
        result_capability: &PromiseCapability,
        promise_resolve: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        #[derive(Debug, Trace, Finalize)]
        struct ResolveElementCaptures {
            #[unsafe_ignore_trace]
            already_called: Rc<Cell<bool>>,
            index: usize,
            values: Gc<GcCell<Vec<JsValue>>>,
            capability_resolve: JsFunction,
            #[unsafe_ignore_trace]
            remaining_elements_count: Rc<Cell<i32>>,
        }

        // 1. Let values be a new empty List.
        let values = Gc::new(GcCell::new(Vec::new()));

        // 2. Let remainingElementsCount be the Record { [[Value]]: 1 }.
        let remaining_elements_count = Rc::new(Cell::new(1));

        // 3. Let index be 0.
        let mut index = 0;

        // 4. Repeat,
        loop {
            // a. Let next be Completion(IteratorStep(iteratorRecord)).
            let next = iterator_record.step(context);

            let next_value = match next {
                Err(e) => {
                    // b. If next is an abrupt completion, set iteratorRecord.[[Done]] to true.
                    iterator_record.set_done(true);

                    // c. ReturnIfAbrupt(next).
                    return Err(e);
                }
                // d. If next is false, then
                Ok(None) => {
                    // i. Set iteratorRecord.[[Done]] to true.
                    iterator_record.set_done(true);

                    // ii. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                    remaining_elements_count.set(remaining_elements_count.get() - 1);

                    // iii. If remainingElementsCount.[[Value]] is 0, then
                    if remaining_elements_count.get() == 0 {
                        // 1. Let valuesArray be CreateArrayFromList(values).
                        let values_array = crate::builtins::Array::create_array_from_list(
                            values.borrow().iter().cloned(),
                            context,
                        );

                        // 2. Perform ? Call(resultCapability.[[Resolve]], undefined, « valuesArray »).
                        result_capability.resolve.call(
                            &JsValue::undefined(),
                            &[values_array.into()],
                            context,
                        )?;
                    }

                    // iv. Return resultCapability.[[Promise]].
                    return Ok(result_capability.promise.clone());
                }
                Ok(Some(next)) => {
                    // e. Let nextValue be Completion(IteratorValue(next)).
                    let next_value = next.value(context);

                    match next_value {
                        Err(e) => {
                            // f. If nextValue is an abrupt completion, set iteratorRecord.[[Done]] to true.
                            iterator_record.set_done(true);

                            // g. ReturnIfAbrupt(nextValue).
                            return Err(e);
                        }
                        Ok(next_value) => next_value,
                    }
                }
            };

            // h. Append undefined to values.
            values.borrow_mut().push(JsValue::Undefined);

            // i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).
            let next_promise =
                promise_resolve.call(&constructor.clone().into(), &[next_value], context)?;

            // j. Let steps be the algorithm steps defined in Promise.all Resolve Element Functions.
            // k. Let length be the number of non-optional parameters of the function definition in Promise.all Resolve Element Functions.
            // l. Let onFulfilled be CreateBuiltinFunction(steps, length, "", « [[AlreadyCalled]], [[Index]], [[Values]], [[Capability]], [[RemainingElements]] »).
            // m. Set onFulfilled.[[AlreadyCalled]] to false.
            // n. Set onFulfilled.[[Index]] to index.
            // o. Set onFulfilled.[[Values]] to values.
            // p. Set onFulfilled.[[Capability]] to resultCapability.
            // q. Set onFulfilled.[[RemainingElements]] to remainingElementsCount.
            let on_fulfilled = FunctionBuilder::closure_with_captures(
                context,
                |_, args, captures, context| {
                    // https://tc39.es/ecma262/#sec-promise.all-resolve-element-functions

                    // 1. Let F be the active function object.
                    // 2. If F.[[AlreadyCalled]] is true, return undefined.
                    if captures.already_called.get() {
                        return Ok(JsValue::undefined());
                    }

                    // 3. Set F.[[AlreadyCalled]] to true.
                    captures.already_called.set(true);

                    // 4. Let index be F.[[Index]].
                    // 5. Let values be F.[[Values]].
                    // 6. Let promiseCapability be F.[[Capability]].
                    // 7. Let remainingElementsCount be F.[[RemainingElements]].

                    // 8. Set values[index] to x.
                    captures.values.borrow_mut()[captures.index] = args.get_or_undefined(0).clone();

                    // 9. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                    captures
                        .remaining_elements_count
                        .set(captures.remaining_elements_count.get() - 1);

                    // 10. If remainingElementsCount.[[Value]] is 0, then
                    if captures.remaining_elements_count.get() == 0 {
                        // a. Let valuesArray be CreateArrayFromList(values).
                        let values_array = crate::builtins::Array::create_array_from_list(
                            captures.values.borrow().as_slice().iter().cloned(),
                            context,
                        );

                        // b. Return ? Call(promiseCapability.[[Resolve]], undefined, « valuesArray »).
                        return captures.capability_resolve.call(
                            &JsValue::undefined(),
                            &[values_array.into()],
                            context,
                        );
                    }

                    // 11. Return undefined.
                    Ok(JsValue::undefined())
                },
                ResolveElementCaptures {
                    already_called: Rc::new(Cell::new(false)),
                    index,
                    values: values.clone(),
                    capability_resolve: result_capability.resolve.clone(),
                    remaining_elements_count: remaining_elements_count.clone(),
                },
            )
            .name("")
            .length(1)
            .constructor(false)
            .build();

            // r. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] + 1.
            remaining_elements_count.set(remaining_elements_count.get() + 1);

            // s. Perform ? Invoke(nextPromise, "then", « onFulfilled, resultCapability.[[Reject]] »).
            next_promise.invoke(
                "then",
                &[on_fulfilled.into(), result_capability.reject.clone().into()],
                context,
            )?;

            // t. Set index to index + 1.
            index += 1;
        }
    }

    /// `Promise.allSettled ( iterable )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.allsettled
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/allSettled
    pub(crate) fn all_settled(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let C be the this value.
        let c = this;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(c, context)?;

        // Note: We already checked that `C` is a constructor in `NewPromiseCapability(C)`.
        let c = c.as_object().expect("must be a constructor");

        // 3. Let promiseResolve be Completion(GetPromiseResolve(C)).
        let promise_resolve = Self::get_promise_resolve(c, context);

        // 4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
        if_abrupt_reject_promise!(promise_resolve, promise_capability, context);

        // 5. Let iteratorRecord be Completion(GetIterator(iterable)).
        let iterator_record = args.get_or_undefined(0).get_iterator(context, None, None);

        // 6. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
        if_abrupt_reject_promise!(iterator_record, promise_capability, context);
        let mut iterator_record = iterator_record;

        // 7. Let result be Completion(PerformPromiseAllSettled(iteratorRecord, C, promiseCapability, promiseResolve)).
        let mut result = Self::perform_promise_all_settled(
            &mut iterator_record,
            c,
            &promise_capability,
            &promise_resolve,
            context,
        )
        .map(JsValue::from);

        // 8. If result is an abrupt completion, then
        if result.is_err() {
            // a. If iteratorRecord.[[Done]] is false, set result to Completion(IteratorClose(iteratorRecord, result)).
            if !iterator_record.done() {
                result = iterator_record.close(result, context);
            }

            // b. IfAbruptRejectPromise(result, promiseCapability).
            if_abrupt_reject_promise!(result, promise_capability, context);

            return Ok(result);
        }

        // 9. Return ? result.
        result
    }

    /// `PerformPromiseAllSettled ( iteratorRecord, constructor, resultCapability, promiseResolve )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromiseallsettled
    fn perform_promise_all_settled(
        iterator_record: &mut IteratorRecord,
        constructor: &JsObject,
        result_capability: &PromiseCapability,
        promise_resolve: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        #[derive(Debug, Trace, Finalize)]
        struct ResolveRejectElementCaptures {
            #[unsafe_ignore_trace]
            already_called: Rc<Cell<bool>>,
            index: usize,
            values: Gc<GcCell<Vec<JsValue>>>,
            capability: JsFunction,
            #[unsafe_ignore_trace]
            remaining_elements: Rc<Cell<i32>>,
        }

        // 1. Let values be a new empty List.
        let values = Gc::new(GcCell::new(Vec::new()));

        // 2. Let remainingElementsCount be the Record { [[Value]]: 1 }.
        let remaining_elements_count = Rc::new(Cell::new(1));

        // 3. Let index be 0.
        let mut index = 0;

        // 4. Repeat,
        loop {
            // a. Let next be Completion(IteratorStep(iteratorRecord)).
            let next = iterator_record.step(context);

            let next_value = match next {
                Err(e) => {
                    // b. If next is an abrupt completion, set iteratorRecord.[[Done]] to true.
                    iterator_record.set_done(true);

                    // c. ReturnIfAbrupt(next).
                    return Err(e);
                }
                // d. If next is false, then
                Ok(None) => {
                    // i. Set iteratorRecord.[[Done]] to true.
                    iterator_record.set_done(true);

                    // ii. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                    remaining_elements_count.set(remaining_elements_count.get() - 1);

                    // iii. If remainingElementsCount.[[Value]] is 0, then
                    if remaining_elements_count.get() == 0 {
                        // 1. Let valuesArray be CreateArrayFromList(values).
                        let values_array = crate::builtins::Array::create_array_from_list(
                            values.borrow().as_slice().iter().cloned(),
                            context,
                        );

                        // 2. Perform ? Call(resultCapability.[[Resolve]], undefined, « valuesArray »).
                        result_capability.resolve.call(
                            &JsValue::undefined(),
                            &[values_array.into()],
                            context,
                        )?;
                    }

                    // iv. Return resultCapability.[[Promise]].
                    return Ok(result_capability.promise.clone());
                }
                Ok(Some(next)) => {
                    // e. Let nextValue be Completion(IteratorValue(next)).
                    let next_value = next.value(context);

                    match next_value {
                        Err(e) => {
                            // f. If nextValue is an abrupt completion, set iteratorRecord.[[Done]] to true.
                            iterator_record.set_done(true);

                            // g. ReturnIfAbrupt(nextValue).
                            return Err(e);
                        }
                        Ok(next_value) => next_value,
                    }
                }
            };

            // h. Append undefined to values.
            values.borrow_mut().push(JsValue::undefined());

            // i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).
            let next_promise =
                promise_resolve.call(&constructor.clone().into(), &[next_value], context)?;

            // j. Let stepsFulfilled be the algorithm steps defined in Promise.allSettled Resolve Element Functions.
            // k. Let lengthFulfilled be the number of non-optional parameters of the function definition in Promise.allSettled Resolve Element Functions.
            // l. Let onFulfilled be CreateBuiltinFunction(stepsFulfilled, lengthFulfilled, "", « [[AlreadyCalled]], [[Index]], [[Values]], [[Capability]], [[RemainingElements]] »).
            // m. Let alreadyCalled be the Record { [[Value]]: false }.
            // n. Set onFulfilled.[[AlreadyCalled]] to alreadyCalled.
            // o. Set onFulfilled.[[Index]] to index.
            // p. Set onFulfilled.[[Values]] to values.
            // q. Set onFulfilled.[[Capability]] to resultCapability.
            // r. Set onFulfilled.[[RemainingElements]] to remainingElementsCount.
            let on_fulfilled = FunctionBuilder::closure_with_captures(
                context,
                |_, args, captures, context| {
                    // https://tc39.es/ecma262/#sec-promise.allsettled-resolve-element-functions

                    // 1. Let F be the active function object.
                    // 2. Let alreadyCalled be F.[[AlreadyCalled]].

                    // 3. If alreadyCalled.[[Value]] is true, return undefined.
                    if captures.already_called.get() {
                        return Ok(JsValue::undefined());
                    }

                    // 4. Set alreadyCalled.[[Value]] to true.
                    captures.already_called.set(true);

                    // 5. Let index be F.[[Index]].
                    // 6. Let values be F.[[Values]].
                    // 7. Let promiseCapability be F.[[Capability]].
                    // 8. Let remainingElementsCount be F.[[RemainingElements]].

                    // 9. Let obj be OrdinaryObjectCreate(%Object.prototype%).
                    let obj = context.construct_object();

                    // 10. Perform ! CreateDataPropertyOrThrow(obj, "status", "fulfilled").
                    obj.create_data_property_or_throw("status", "fulfilled", context)
                        .expect("cannot fail per spec");

                    // 11. Perform ! CreateDataPropertyOrThrow(obj, "value", x).
                    obj.create_data_property_or_throw("value", args.get_or_undefined(0), context)
                        .expect("cannot fail per spec");

                    // 12. Set values[index] to obj.
                    captures.values.borrow_mut()[captures.index] = obj.into();

                    // 13. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                    captures
                        .remaining_elements
                        .set(captures.remaining_elements.get() - 1);

                    // 14. If remainingElementsCount.[[Value]] is 0, then
                    if captures.remaining_elements.get() == 0 {
                        // a. Let valuesArray be CreateArrayFromList(values).
                        let values_array = Array::create_array_from_list(
                            captures.values.borrow().as_slice().iter().cloned(),
                            context,
                        );

                        // b. Return ? Call(promiseCapability.[[Resolve]], undefined, « valuesArray »).
                        return captures.capability.call(
                            &JsValue::undefined(),
                            &[values_array.into()],
                            context,
                        );
                    }

                    // 15. Return undefined.
                    Ok(JsValue::undefined())
                },
                ResolveRejectElementCaptures {
                    already_called: Rc::new(Cell::new(false)),
                    index,
                    values: values.clone(),
                    capability: result_capability.resolve.clone(),
                    remaining_elements: remaining_elements_count.clone(),
                },
            )
            .name("")
            .length(1)
            .constructor(false)
            .build();

            // s. Let stepsRejected be the algorithm steps defined in Promise.allSettled Reject Element Functions.
            // t. Let lengthRejected be the number of non-optional parameters of the function definition in Promise.allSettled Reject Element Functions.
            // u. Let onRejected be CreateBuiltinFunction(stepsRejected, lengthRejected, "", « [[AlreadyCalled]], [[Index]], [[Values]], [[Capability]], [[RemainingElements]] »).
            // v. Set onRejected.[[AlreadyCalled]] to alreadyCalled.
            // w. Set onRejected.[[Index]] to index.
            // x. Set onRejected.[[Values]] to values.
            // y. Set onRejected.[[Capability]] to resultCapability.
            // z. Set onRejected.[[RemainingElements]] to remainingElementsCount.
            let on_rejected = FunctionBuilder::closure_with_captures(
                context,
                |_, args, captures, context| {
                    // https://tc39.es/ecma262/#sec-promise.allsettled-reject-element-functions

                    // 1. Let F be the active function object.
                    // 2. Let alreadyCalled be F.[[AlreadyCalled]].

                    // 3. If alreadyCalled.[[Value]] is true, return undefined.
                    if captures.already_called.get() {
                        return Ok(JsValue::undefined());
                    }

                    // 4. Set alreadyCalled.[[Value]] to true.
                    captures.already_called.set(true);

                    // 5. Let index be F.[[Index]].
                    // 6. Let values be F.[[Values]].
                    // 7. Let promiseCapability be F.[[Capability]].
                    // 8. Let remainingElementsCount be F.[[RemainingElements]].

                    // 9. Let obj be OrdinaryObjectCreate(%Object.prototype%).
                    let obj = context.construct_object();

                    // 10. Perform ! CreateDataPropertyOrThrow(obj, "status", "rejected").
                    obj.create_data_property_or_throw("status", "rejected", context)
                        .expect("cannot fail per spec");

                    // 11. Perform ! CreateDataPropertyOrThrow(obj, "reason", x).
                    obj.create_data_property_or_throw("reason", args.get_or_undefined(0), context)
                        .expect("cannot fail per spec");

                    // 12. Set values[index] to obj.
                    captures.values.borrow_mut()[captures.index] = obj.into();

                    // 13. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                    captures
                        .remaining_elements
                        .set(captures.remaining_elements.get() - 1);

                    // 14. If remainingElementsCount.[[Value]] is 0, then
                    if captures.remaining_elements.get() == 0 {
                        // a. Let valuesArray be CreateArrayFromList(values).
                        let values_array = Array::create_array_from_list(
                            captures.values.borrow().as_slice().iter().cloned(),
                            context,
                        );

                        // b. Return ? Call(promiseCapability.[[Resolve]], undefined, « valuesArray »).
                        return captures.capability.call(
                            &JsValue::undefined(),
                            &[values_array.into()],
                            context,
                        );
                    }

                    // 15. Return undefined.
                    Ok(JsValue::undefined())
                },
                ResolveRejectElementCaptures {
                    already_called: Rc::new(Cell::new(false)),
                    index,
                    values: values.clone(),
                    capability: result_capability.resolve.clone(),
                    remaining_elements: remaining_elements_count.clone(),
                },
            )
            .name("")
            .length(1)
            .constructor(false)
            .build();

            // aa. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] + 1.
            remaining_elements_count.set(remaining_elements_count.get() + 1);

            // ab. Perform ? Invoke(nextPromise, "then", « onFulfilled, onRejected »).
            next_promise.invoke("then", &[on_fulfilled.into(), on_rejected.into()], context)?;

            // ac. Set index to index + 1.
            index += 1;
        }
    }

    /// `Promise.any ( iterable )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.any
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/any
    pub(crate) fn any(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let C be the this value.
        let c = this;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(c, context)?;

        // Note: We already checked that `C` is a constructor in `NewPromiseCapability(C)`.
        let c = c.as_object().expect("must be a constructor");

        // 3. Let promiseResolve be Completion(GetPromiseResolve(C)).
        let promise_resolve = Self::get_promise_resolve(c, context);

        // 4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
        if_abrupt_reject_promise!(promise_resolve, promise_capability, context);

        // 5. Let iteratorRecord be Completion(GetIterator(iterable)).
        let iterator_record = args.get_or_undefined(0).get_iterator(context, None, None);

        // 6. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
        if_abrupt_reject_promise!(iterator_record, promise_capability, context);
        let mut iterator_record = iterator_record;

        // 7. Let result be Completion(PerformPromiseAny(iteratorRecord, C, promiseCapability, promiseResolve)).
        let mut result = Self::perform_promise_any(
            &mut iterator_record,
            c,
            &promise_capability,
            &promise_resolve,
            context,
        )
        .map(JsValue::from);

        // 8. If result is an abrupt completion, then
        if result.is_err() {
            // a. If iteratorRecord.[[Done]] is false, set result to Completion(IteratorClose(iteratorRecord, result)).
            if !iterator_record.done() {
                result = iterator_record.close(result, context);
            }

            // b. IfAbruptRejectPromise(result, promiseCapability).
            if_abrupt_reject_promise!(result, promise_capability, context);

            return Ok(result);
        }

        // 9. Return ? result.
        result
    }

    /// `PerformPromiseAny ( iteratorRecord, constructor, resultCapability, promiseResolve )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromiseany
    fn perform_promise_any(
        iterator_record: &mut IteratorRecord,
        constructor: &JsObject,
        result_capability: &PromiseCapability,
        promise_resolve: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        #[derive(Debug, Trace, Finalize)]
        struct RejectElementCaptures {
            #[unsafe_ignore_trace]
            already_called: Rc<Cell<bool>>,
            index: usize,
            errors: Gc<GcCell<Vec<JsValue>>>,
            capability_reject: JsFunction,
            #[unsafe_ignore_trace]
            remaining_elements_count: Rc<Cell<i32>>,
        }

        // 1. Let errors be a new empty List.
        let errors = Gc::new(GcCell::new(Vec::new()));

        // 2. Let remainingElementsCount be the Record { [[Value]]: 1 }.
        let remaining_elements_count = Rc::new(Cell::new(1));

        // 3. Let index be 0.
        let mut index = 0;

        // 4. Repeat,
        loop {
            // a. Let next be Completion(IteratorStep(iteratorRecord)).
            let next = iterator_record.step(context);

            let next_value = match next {
                Err(e) => {
                    // b. If next is an abrupt completion, set iteratorRecord.[[Done]] to true.
                    iterator_record.set_done(true);

                    // c. ReturnIfAbrupt(next).
                    return Err(e);
                }
                // d. If next is false, then
                Ok(None) => {
                    // i. Set iteratorRecord.[[Done]] to true.
                    iterator_record.set_done(true);

                    // ii. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                    remaining_elements_count.set(remaining_elements_count.get() - 1);

                    // iii. If remainingElementsCount.[[Value]] is 0, then
                    if remaining_elements_count.get() == 0 {
                        // 1. Let error be a newly created AggregateError object.
                        let error = JsObject::from_proto_and_data(
                            context
                                .intrinsics()
                                .constructors()
                                .aggregate_error()
                                .prototype(),
                            ObjectData::error(ErrorKind::Aggregate),
                        );

                        // 2. Perform ! DefinePropertyOrThrow(error, "errors", PropertyDescriptor { [[Configurable]]: true, [[Enumerable]]: false, [[Writable]]: true, [[Value]]: CreateArrayFromList(errors) }).
                        error
                            .define_property_or_throw(
                                "errors",
                                PropertyDescriptorBuilder::new()
                                    .configurable(true)
                                    .enumerable(false)
                                    .writable(true)
                                    .value(Array::create_array_from_list(
                                        errors.borrow().as_slice().iter().cloned(),
                                        context,
                                    )),
                                context,
                            )
                            .expect("cannot fail per spec");

                        // 3. Return ThrowCompletion(error).
                        return Err(JsError::from_opaque(error.into()));
                    }

                    // iv. Return resultCapability.[[Promise]].
                    return Ok(result_capability.promise.clone());
                }
                Ok(Some(next)) => {
                    // e. Let nextValue be Completion(IteratorValue(next)).
                    let next_value = next.value(context);

                    match next_value {
                        Err(e) => {
                            // f. If nextValue is an abrupt completion, set iteratorRecord.[[Done]] to true.
                            iterator_record.set_done(true);

                            // g. ReturnIfAbrupt(nextValue).
                            return Err(e);
                        }
                        Ok(next_value) => next_value,
                    }
                }
            };

            // h. Append undefined to errors.
            errors.borrow_mut().push(JsValue::undefined());

            // i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).
            let next_promise =
                promise_resolve.call(&constructor.clone().into(), &[next_value], context)?;

            // j. Let stepsRejected be the algorithm steps defined in Promise.any Reject Element Functions.
            // k. Let lengthRejected be the number of non-optional parameters of the function definition in Promise.any Reject Element Functions.
            // l. Let onRejected be CreateBuiltinFunction(stepsRejected, lengthRejected, "", « [[AlreadyCalled]], [[Index]], [[Errors]], [[Capability]], [[RemainingElements]] »).
            // m. Set onRejected.[[AlreadyCalled]] to false.
            // n. Set onRejected.[[Index]] to index.
            // o. Set onRejected.[[Errors]] to errors.
            // p. Set onRejected.[[Capability]] to resultCapability.
            // q. Set onRejected.[[RemainingElements]] to remainingElementsCount.
            let on_rejected = FunctionBuilder::closure_with_captures(
                context,
                |_, args, captures, context| {
                    // https://tc39.es/ecma262/#sec-promise.any-reject-element-functions

                    // 1. Let F be the active function object.

                    // 2. If F.[[AlreadyCalled]] is true, return undefined.
                    if captures.already_called.get() {
                        return Ok(JsValue::undefined());
                    }

                    // 3. Set F.[[AlreadyCalled]] to true.
                    captures.already_called.set(true);

                    // 4. Let index be F.[[Index]].
                    // 5. Let errors be F.[[Errors]].
                    // 6. Let promiseCapability be F.[[Capability]].
                    // 7. Let remainingElementsCount be F.[[RemainingElements]].

                    // 8. Set errors[index] to x.
                    captures.errors.borrow_mut()[captures.index] = args.get_or_undefined(0).clone();

                    // 9. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                    captures
                        .remaining_elements_count
                        .set(captures.remaining_elements_count.get() - 1);

                    // 10. If remainingElementsCount.[[Value]] is 0, then
                    if captures.remaining_elements_count.get() == 0 {
                        // a. Let error be a newly created AggregateError object.
                        let error = JsObject::from_proto_and_data(
                            context
                                .intrinsics()
                                .constructors()
                                .aggregate_error()
                                .prototype(),
                            ObjectData::error(ErrorKind::Aggregate),
                        );

                        // b. Perform ! DefinePropertyOrThrow(error, "errors", PropertyDescriptor { [[Configurable]]: true, [[Enumerable]]: false, [[Writable]]: true, [[Value]]: CreateArrayFromList(errors) }).
                        error
                            .define_property_or_throw(
                                "errors",
                                PropertyDescriptorBuilder::new()
                                    .configurable(true)
                                    .enumerable(false)
                                    .writable(true)
                                    .value(Array::create_array_from_list(
                                        captures.errors.borrow().as_slice().iter().cloned(),
                                        context,
                                    )),
                                context,
                            )
                            .expect("cannot fail per spec");
                        // c. Return ? Call(promiseCapability.[[Reject]], undefined, « error »).
                        return captures.capability_reject.call(
                            &JsValue::undefined(),
                            &[error.into()],
                            context,
                        );
                    }

                    // 11. Return undefined.
                    Ok(JsValue::undefined())
                },
                RejectElementCaptures {
                    already_called: Rc::new(Cell::new(false)),
                    index,
                    errors: errors.clone(),
                    capability_reject: result_capability.reject.clone(),
                    remaining_elements_count: remaining_elements_count.clone(),
                },
            )
            .name("")
            .length(1)
            .constructor(false)
            .build();

            // r. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] + 1.
            remaining_elements_count.set(remaining_elements_count.get() + 1);

            // s. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], onRejected »).
            next_promise.invoke(
                "then",
                &[result_capability.resolve.clone().into(), on_rejected.into()],
                context,
            )?;

            // t. Set index to index + 1.
            index += 1;
        }
    }

    /// `CreateResolvingFunctions ( promise )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createresolvingfunctions
    fn create_resolving_functions(
        promise: &JsObject,
        context: &mut Context,
    ) -> ResolvingFunctionsRecord {
        #[derive(Debug, Trace, Finalize)]
        struct RejectResolveCaptures {
            promise: JsObject,
            #[unsafe_ignore_trace]
            already_resolved: Rc<Cell<bool>>,
        }

        // 1. Let alreadyResolved be the Record { [[Value]]: false }.
        let already_resolved = Rc::new(Cell::new(false));

        // 5. Set resolve.[[Promise]] to promise.
        // 6. Set resolve.[[AlreadyResolved]] to alreadyResolved.
        let resolve_captures = RejectResolveCaptures {
            already_resolved: already_resolved.clone(),
            promise: promise.clone(),
        };

        // 2. Let stepsResolve be the algorithm steps defined in Promise Resolve Functions.
        // 3. Let lengthResolve be the number of non-optional parameters of the function definition in Promise Resolve Functions.
        // 4. Let resolve be CreateBuiltinFunction(stepsResolve, lengthResolve, "", « [[Promise]], [[AlreadyResolved]] »).
        let resolve = FunctionBuilder::closure_with_captures(
            context,
            |_this, args, captures, context| {
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
                if already_resolved.get() {
                    return Ok(JsValue::Undefined);
                }

                // 6. Set alreadyResolved.[[Value]] to true.
                already_resolved.set(true);

                let resolution = args.get_or_undefined(0);

                // 7. If SameValue(resolution, promise) is true, then
                if JsValue::same_value(resolution, &promise.clone().into()) {
                    //   a. Let selfResolutionError be a newly created TypeError object.
                    let self_resolution_error = JsNativeError::typ()
                        .with_message("SameValue(resolution, promise) is true")
                        .to_opaque(context);

                    //   b. Perform RejectPromise(promise, selfResolutionError).
                    promise
                        .borrow_mut()
                        .as_promise_mut()
                        .expect("Expected promise to be a Promise")
                        .reject_promise(&self_resolution_error.into(), context);

                    //   c. Return undefined.
                    return Ok(JsValue::Undefined);
                }

                let Some(then) = resolution.as_object() else {
                    // 8. If Type(resolution) is not Object, then
                    //   a. Perform FulfillPromise(promise, resolution).
                    promise
                    .borrow_mut()
                    .as_promise_mut()
                    .expect("Expected promise to be a Promise")
                    .fulfill_promise(resolution, context)?;

                    //   b. Return undefined.
                    return Ok(JsValue::Undefined);
                };

                // 9. Let then be Completion(Get(resolution, "then")).
                let then_action = match then.get("then", context) {
                    // 10. If then is an abrupt completion, then
                    Err(e) => {
                        //   a. Perform RejectPromise(promise, then.[[Value]]).
                        promise
                            .borrow_mut()
                            .as_promise_mut()
                            .expect("Expected promise to be a Promise")
                            .reject_promise(&e.to_opaque(context), context);

                        //   b. Return undefined.
                        return Ok(JsValue::Undefined);
                    }
                    // 11. Let thenAction be then.[[Value]].
                    Ok(then) => then,
                };

                // 12. If IsCallable(thenAction) is false, then
                let then_action = match then_action.as_object() {
                    Some(then_action) if then_action.is_callable() => then_action,
                    _ => {
                        // a. Perform FulfillPromise(promise, resolution).
                        promise
                            .borrow_mut()
                            .as_promise_mut()
                            .expect("Expected promise to be a Promise")
                            .fulfill_promise(resolution, context)?;

                        //   b. Return undefined.
                        return Ok(JsValue::Undefined);
                    }
                };

                // 13. Let thenJobCallback be HostMakeJobCallback(thenAction).
                let then_job_callback = JobCallback::make_job_callback(then_action.clone());

                // 14. Let job be NewPromiseResolveThenableJob(promise, resolution, thenJobCallback).
                let job: JobCallback = PromiseJob::new_promise_resolve_thenable_job(
                    promise.clone(),
                    resolution.clone(),
                    then_job_callback,
                    context,
                );

                // 15. Perform HostEnqueuePromiseJob(job.[[Job]], job.[[Realm]]).
                context.host_enqueue_promise_job(job);

                // 16. Return undefined.
                Ok(JsValue::Undefined)
            },
            resolve_captures,
        )
        .name("")
        .length(1)
        .constructor(false)
        .build();

        // 10. Set reject.[[Promise]] to promise.
        // 11. Set reject.[[AlreadyResolved]] to alreadyResolved.
        let reject_captures = RejectResolveCaptures {
            promise: promise.clone(),
            already_resolved,
        };

        // 7. Let stepsReject be the algorithm steps defined in Promise Reject Functions.
        // 8. Let lengthReject be the number of non-optional parameters of the function definition in Promise Reject Functions.
        // 9. Let reject be CreateBuiltinFunction(stepsReject, lengthReject, "", « [[Promise]], [[AlreadyResolved]] »).
        let reject = FunctionBuilder::closure_with_captures(
            context,
            |_this, args, captures, context| {
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
                if already_resolved.get() {
                    return Ok(JsValue::Undefined);
                }

                // 6. Set alreadyResolved.[[Value]] to true.
                already_resolved.set(true);

                // let reason = args.get_or_undefined(0);
                // 7. Perform RejectPromise(promise, reason).
                promise
                    .borrow_mut()
                    .as_promise_mut()
                    .expect("Expected promise to be a Promise")
                    .reject_promise(args.get_or_undefined(0), context);

                // 8. Return undefined.
                Ok(JsValue::Undefined)
            },
            reject_captures,
        )
        .name("")
        .length(1)
        .constructor(false)
        .build();

        // 12. Return the Record { [[Resolve]]: resolve, [[Reject]]: reject }.
        let resolve = resolve.conv::<JsValue>();
        let reject = reject.conv::<JsValue>();
        ResolvingFunctionsRecord { resolve, reject }
    }

    /// `FulfillPromise ( promise, value )`
    ///
    /// The abstract operation `FulfillPromise` takes arguments `promise` and `value` and returns
    /// `unused`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-fulfillpromise
    pub fn fulfill_promise(&mut self, value: &JsValue, context: &mut Context) -> JsResult<()> {
        // 1. Assert: The value of promise.[[PromiseState]] is pending.
        assert!(
            matches!(self.promise_state, PromiseState::Pending),
            "promise was not pending"
        );

        // 2. Let reactions be promise.[[PromiseFulfillReactions]].
        let reactions = &self.promise_fulfill_reactions;

        // 7. Perform TriggerPromiseReactions(reactions, value).
        Self::trigger_promise_reactions(reactions, value, context);
        // reordering this statement does not affect the semantics

        // 4. Set promise.[[PromiseFulfillReactions]] to undefined.
        self.promise_fulfill_reactions = Vec::new();

        // 5. Set promise.[[PromiseRejectReactions]] to undefined.
        self.promise_reject_reactions = Vec::new();

        // 3. Set promise.[[PromiseResult]] to value.
        // 6. Set promise.[[PromiseState]] to fulfilled.
        self.promise_state = PromiseState::Fulfilled(value.clone());

        // 8. Return unused.
        Ok(())
    }

    /// `RejectPromise ( promise, reason )`
    ///
    /// The abstract operation `RejectPromise` takes arguments `promise` and `reason` and returns
    /// `unused`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-rejectpromise
    pub fn reject_promise(&mut self, reason: &JsValue, context: &mut Context) {
        // 1. Assert: The value of promise.[[PromiseState]] is pending.
        assert!(
            matches!(self.promise_state, PromiseState::Pending),
            "Expected promise.[[PromiseState]] to be pending"
        );

        // 2. Let reactions be promise.[[PromiseRejectReactions]].
        let reactions = &self.promise_reject_reactions;

        // 8. Perform TriggerPromiseReactions(reactions, reason).
        Self::trigger_promise_reactions(reactions, reason, context);
        // reordering this statement does not affect the semantics

        // 4. Set promise.[[PromiseFulfillReactions]] to undefined.
        self.promise_fulfill_reactions = Vec::new();

        // 5. Set promise.[[PromiseRejectReactions]] to undefined.
        self.promise_reject_reactions = Vec::new();

        // 3. Set promise.[[PromiseResult]] to reason.
        // 6. Set promise.[[PromiseState]] to rejected.
        self.promise_state = PromiseState::Rejected(reason.clone());

        // 7. If promise.[[PromiseIsHandled]] is false, perform HostPromiseRejectionTracker(promise, "reject").
        if !self.promise_is_handled {
            // TODO
        }

        // 9. Return unused.
    }

    /// `TriggerPromiseReactions ( reactions, argument )`
    ///
    /// The abstract operation `TriggerPromiseReactions` takes arguments `reactions` (a `List` of
    /// `PromiseReaction` Records) and `argument` and returns unused. It enqueues a new `Job` for
    /// each record in `reactions`. Each such `Job` processes the `[[Type]]` and `[[Handler]]` of
    /// the `PromiseReaction` Record, and if the `[[Handler]]` is not `empty`, calls it passing the
    /// given argument. If the `[[Handler]]` is `empty`, the behaviour is determined by the
    /// `[[Type]]`.
    ///
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
            context.host_enqueue_promise_job(job);
        }

        // 2. Return unused.
    }

    /// `Promise.race ( iterable )`
    ///
    /// The `race` function returns a new promise which is settled in the same way as the first
    /// passed promise to settle. It resolves all elements of the passed `iterable` to promises.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.race
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/race
    pub fn race(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let iterable = args.get_or_undefined(0);

        // 1. Let C be the this value.
        let c = this;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(c, context)?;

        // 3. Let promiseResolve be Completion(GetPromiseResolve(C)).
        let promise_resolve =
            Self::get_promise_resolve(c.as_object().expect("this was not an object"), context);

        // 4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
        if_abrupt_reject_promise!(promise_resolve, promise_capability, context);

        // 5. Let iteratorRecord be Completion(GetIterator(iterable)).
        let iterator_record = iterable.get_iterator(context, None, None);

        // 6. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
        if_abrupt_reject_promise!(iterator_record, promise_capability, context);
        let mut iterator_record = iterator_record;

        // 7. Let result be Completion(PerformPromiseRace(iteratorRecord, C, promiseCapability, promiseResolve)).
        let mut result = Self::perform_promise_race(
            &mut iterator_record,
            c,
            &promise_capability,
            &promise_resolve.into(),
            context,
        );

        // 8. If result is an abrupt completion, then
        if result.is_err() {
            // a. If iteratorRecord.[[Done]] is false, set result to Completion(IteratorClose(iteratorRecord, result)).
            if !iterator_record.done() {
                result = iterator_record.close(result, context);
            }

            // b. IfAbruptRejectPromise(result, promiseCapability).
            if_abrupt_reject_promise!(result, promise_capability, context);

            Ok(result)
        } else {
            // 9. Return ? result.
            result
        }
    }

    /// `PerformPromiseRace ( iteratorRecord, constructor, resultCapability, promiseResolve )`
    ///
    /// The abstract operation `PerformPromiseRace` takes arguments `iteratorRecord`, `constructor`
    /// (a constructor), `resultCapability` (a [`PromiseCapability`] Record), and `promiseResolve`
    /// (a function object) and returns either a normal completion containing an ECMAScript
    /// language value or a throw completion.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromiserace
    fn perform_promise_race(
        iterator_record: &mut IteratorRecord,
        constructor: &JsValue,
        result_capability: &PromiseCapability,
        promise_resolve: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Repeat,
        loop {
            // a. Let next be Completion(IteratorStep(iteratorRecord)).
            let next = iterator_record.step(context);

            // b. If next is an abrupt completion, set iteratorRecord.[[Done]] to true.
            if next.is_err() {
                iterator_record.set_done(true);
            }

            // c. ReturnIfAbrupt(next).
            let next = next?;

            if let Some(next) = next {
                // e. Let nextValue be Completion(IteratorValue(next)).
                let next_value = next.value(context);

                // f. If nextValue is an abrupt completion, set iteratorRecord.[[Done]] to true.
                if next_value.is_err() {
                    iterator_record.set_done(true);
                }

                // g. ReturnIfAbrupt(nextValue).
                let next_value = next_value?;

                // h. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).
                let next_promise = context.call(promise_resolve, constructor, &[next_value])?;

                // i. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], resultCapability.[[Reject]] »).
                next_promise.invoke(
                    "then",
                    &[
                        result_capability.resolve.clone().into(),
                        result_capability.reject.clone().into(),
                    ],
                    context,
                )?;
            } else {
                // d. If next is false, then
                // i. Set iteratorRecord.[[Done]] to true.
                iterator_record.set_done(true);

                // ii. Return resultCapability.[[Promise]].
                return Ok(result_capability.promise.clone().into());
            }
        }
    }

    /// `Promise.reject ( r )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.reject
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/reject
    pub fn reject(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let r = args.get_or_undefined(0);

        // 1. Let C be the this value.
        let c = this;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(c, context)?;

        // 3. Perform ? Call(promiseCapability.[[Reject]], undefined, « r »).
        context.call(
            &promise_capability.reject.clone().into(),
            &JsValue::undefined(),
            &[r.clone()],
        )?;

        // 4. Return promiseCapability.[[Promise]].
        Ok(promise_capability.promise.clone().into())
    }

    /// `Promise.resolve ( x )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.resolve
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/resolve
    pub fn resolve(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let x = args.get_or_undefined(0);

        // 1. Let C be the this value.
        let c = this;

        if let Some(c) = c.as_object() {
            // 3. Return ? PromiseResolve(C, x).
            Self::promise_resolve(c.clone(), x.clone(), context)
        } else {
            // 2. If Type(C) is not Object, throw a TypeError exception.
            Err(JsNativeError::typ()
                .with_message("Promise.resolve() called on a non-object")
                .into())
        }
    }

    /// `get Promise [ @@species ]`
    ///
    /// The `Promise [ @@species ]` accessor property returns the Promise constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-promise-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `Promise.prototype.catch ( onRejected )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.prototype.catch
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/catch
    pub fn catch(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let on_rejected = args.get_or_undefined(0);

        // 1. Let promise be the this value.
        let promise = this;
        // 2. Return ? Invoke(promise, "then", « undefined, onRejected »).
        promise.invoke(
            "then",
            &[JsValue::undefined(), on_rejected.clone()],
            context,
        )
    }

    /// `Promise.prototype.finally ( onFinally )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.prototype.finally
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/finally
    pub fn finally(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let promise be the this value.
        let promise = this;

        // 2. If Type(promise) is not Object, throw a TypeError exception.
        let Some(promise_obj) = promise.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("finally called with a non-object promise")
                .into());
        };

        // 3. Let C be ? SpeciesConstructor(promise, %Promise%).
        let c = promise_obj.species_constructor(StandardConstructors::promise, context)?;

        // 4. Assert: IsConstructor(C) is true.
        assert!(c.is_constructor());

        let on_finally = args.get_or_undefined(0);

        // 5. If IsCallable(onFinally) is false, then
        let (then_finally, catch_finally) = if on_finally.is_callable() {
            /// Capture object for the `thenFinallyClosure` abstract closure.
            #[derive(Debug, Trace, Finalize)]
            struct FinallyCaptures {
                on_finally: JsValue,
                c: JsObject,
            }

            // a. Let thenFinallyClosure be a new Abstract Closure with parameters (value) that captures onFinally and C and performs the following steps when called:
            let then_finally_closure = FunctionBuilder::closure_with_captures(
                context,
                |_this, args, captures, context| {
                    /// Capture object for the abstract `returnValue` closure.
                    #[derive(Debug, Trace, Finalize)]
                    struct ReturnValueCaptures {
                        value: JsValue,
                    }

                    let value = args.get_or_undefined(0);

                    // i. Let result be ? Call(onFinally, undefined).
                    let result = context.call(&captures.on_finally, &JsValue::undefined(), &[])?;

                    // ii. Let promise be ? PromiseResolve(C, result).
                    let promise = Self::promise_resolve(captures.c.clone(), result, context)?;

                    // iii. Let returnValue be a new Abstract Closure with no parameters that captures value and performs the following steps when called:
                    let return_value = FunctionBuilder::closure_with_captures(
                        context,
                        |_this, _args, captures, _context| {
                            // 1. Return value.
                            Ok(captures.value.clone())
                        },
                        ReturnValueCaptures {
                            value: value.clone(),
                        },
                    );

                    // iv. Let valueThunk be CreateBuiltinFunction(returnValue, 0, "", « »).
                    let value_thunk = return_value.length(0).name("").build();

                    // v. Return ? Invoke(promise, "then", « valueThunk »).
                    promise.invoke("then", &[value_thunk.into()], context)
                },
                FinallyCaptures {
                    on_finally: on_finally.clone(),
                    c: c.clone(),
                },
            );

            // b. Let thenFinally be CreateBuiltinFunction(thenFinallyClosure, 1, "", « »).
            let then_finally = then_finally_closure.length(1).name("").build();

            // c. Let catchFinallyClosure be a new Abstract Closure with parameters (reason) that captures onFinally and C and performs the following steps when called:
            let catch_finally_closure = FunctionBuilder::closure_with_captures(
                context,
                |_this, args, captures, context| {
                    /// Capture object for the abstract `throwReason` closure.
                    #[derive(Debug, Trace, Finalize)]
                    struct ThrowReasonCaptures {
                        reason: JsValue,
                    }

                    let reason = args.get_or_undefined(0);

                    // i. Let result be ? Call(onFinally, undefined).
                    let result = context.call(&captures.on_finally, &JsValue::undefined(), &[])?;

                    // ii. Let promise be ? PromiseResolve(C, result).
                    let promise = Self::promise_resolve(captures.c.clone(), result, context)?;

                    // iii. Let throwReason be a new Abstract Closure with no parameters that captures reason and performs the following steps when called:
                    let throw_reason = FunctionBuilder::closure_with_captures(
                        context,
                        |_this, _args, captures, _context| {
                            // 1. Return ThrowCompletion(reason).
                            Err(JsError::from_opaque(captures.reason.clone()))
                        },
                        ThrowReasonCaptures {
                            reason: reason.clone(),
                        },
                    );

                    // iv. Let thrower be CreateBuiltinFunction(throwReason, 0, "", « »).
                    let thrower = throw_reason.length(0).name("").build();

                    // v. Return ? Invoke(promise, "then", « thrower »).
                    promise.invoke("then", &[thrower.into()], context)
                },
                FinallyCaptures {
                    on_finally: on_finally.clone(),
                    c,
                },
            );

            // d. Let catchFinally be CreateBuiltinFunction(catchFinallyClosure, 1, "", « »).
            let catch_finally = catch_finally_closure.length(1).name("").build();

            (then_finally.into(), catch_finally.into()) // TODO
        } else {
            // 6. Else,
            //  a. Let thenFinally be onFinally.
            //  b. Let catchFinally be onFinally.
            (on_finally.clone(), on_finally.clone())
        };

        // 7. Return ? Invoke(promise, "then", « thenFinally, catchFinally »).
        promise.invoke("then", &[then_finally, catch_finally], context)
    }

    /// `Promise.prototype.then ( onFulfilled, onRejected )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.prototype.then
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/then
    pub fn then(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let promise be the this value.
        let promise = this;

        // 2. If IsPromise(promise) is false, throw a TypeError exception.
        let promise_obj = match promise.as_promise() {
            Some(obj) => obj,
            None => {
                return Err(JsNativeError::typ()
                    .with_message("IsPromise(promise) is false")
                    .into())
            }
        };

        // 3. Let C be ? SpeciesConstructor(promise, %Promise%).
        let c = promise_obj.species_constructor(StandardConstructors::promise, context)?;

        // 4. Let resultCapability be ? NewPromiseCapability(C).
        let result_capability = PromiseCapability::new(&c.into(), context)?;

        let on_fulfilled = args.get_or_undefined(0);
        let on_rejected = args.get_or_undefined(1);

        // 5. Return PerformPromiseThen(promise, onFulfilled, onRejected, resultCapability).
        promise_obj
            .borrow_mut()
            .as_promise_mut()
            .expect("IsPromise(promise) is false")
            .perform_promise_then(on_fulfilled, on_rejected, Some(result_capability), context)
            .pipe(Ok)
    }

    /// `PerformPromiseThen ( promise, onFulfilled, onRejected [ , resultCapability ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromisethen
    pub(crate) fn perform_promise_then(
        &mut self,
        on_fulfilled: &JsValue,
        on_rejected: &JsValue,
        result_capability: Option<PromiseCapability>,
        context: &mut Context,
    ) -> JsValue {
        // 1. Assert: IsPromise(promise) is true.

        // 2. If resultCapability is not present, then
        //   a. Set resultCapability to undefined.

        let on_fulfilled_job_callback = match on_fulfilled.as_object() {
            // 4. Else,
            //   a. Let onFulfilledJobCallback be HostMakeJobCallback(onFulfilled).
            Some(on_fulfilled) if on_fulfilled.is_callable() => {
                Some(JobCallback::make_job_callback(on_fulfilled.clone()))
            }
            // 3. If IsCallable(onFulfilled) is false, then
            //   a. Let onFulfilledJobCallback be empty.
            _ => None,
        };

        let on_rejected_job_callback = match on_rejected.as_object() {
            // 6. Else,
            //   a. Let onRejectedJobCallback be HostMakeJobCallback(onRejected).
            Some(on_rejected) if on_rejected.is_callable() => {
                Some(JobCallback::make_job_callback(on_rejected.clone()))
            }
            // 5. If IsCallable(onRejected) is false, then
            //   a. Let onRejectedJobCallback be empty.
            _ => None,
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
            //   a. Let value be promise.[[PromiseResult]].
            PromiseState::Fulfilled(ref value) => {
                //   b. Let fulfillJob be NewPromiseReactionJob(fulfillReaction, value).
                let fulfill_job =
                    PromiseJob::new_promise_reaction_job(fulfill_reaction, value.clone(), context);

                //   c. Perform HostEnqueuePromiseJob(fulfillJob.[[Job]], fulfillJob.[[Realm]]).
                context.host_enqueue_promise_job(fulfill_job);
            }

            // 11. Else,
            //   a. Assert: The value of promise.[[PromiseState]] is rejected.
            //   b. Let reason be promise.[[PromiseResult]].
            PromiseState::Rejected(ref reason) => {
                //   c. If promise.[[PromiseIsHandled]] is false, perform HostPromiseRejectionTracker(promise, "handle").
                if !self.promise_is_handled {
                    // HostPromiseRejectionTracker(promise, "handle")
                    // TODO
                }

                //   d. Let rejectJob be NewPromiseReactionJob(rejectReaction, reason).
                let reject_job =
                    PromiseJob::new_promise_reaction_job(reject_reaction, reason.clone(), context);

                //   e. Perform HostEnqueuePromiseJob(rejectJob.[[Job]], rejectJob.[[Realm]]).
                context.host_enqueue_promise_job(reject_job);

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
            Some(result_capability) => result_capability.promise.clone().into(),
        }
    }

    /// `PromiseResolve ( C, x )`
    ///
    /// The abstract operation `PromiseResolve` takes arguments `C` (a constructor) and `x` (an
    /// ECMAScript language value) and returns either a normal completion containing an ECMAScript
    /// language value or a throw completion. It returns a new promise resolved with `x`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise-resolve
    pub(crate) fn promise_resolve(
        c: JsObject,
        x: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If IsPromise(x) is true, then
        if let Some(x) = x.as_promise() {
            // a. Let xConstructor be ? Get(x, "constructor").
            let x_constructor = x.get("constructor", context)?;
            // b. If SameValue(xConstructor, C) is true, return x.
            if JsValue::same_value(&x_constructor, &JsValue::from(c.clone())) {
                return Ok(JsValue::from(x.clone()));
            }
        }

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(&c.into(), context)?;

        // 3. Perform ? Call(promiseCapability.[[Resolve]], undefined, « x »).
        context.call(
            &promise_capability.resolve.clone().into(),
            &JsValue::undefined(),
            &[x],
        )?;

        // 4. Return promiseCapability.[[Promise]].
        Ok(promise_capability.promise.clone().into())
    }

    /// `GetPromiseResolve ( promiseConstructor )`
    ///
    /// The abstract operation `GetPromiseResolve` takes argument `promiseConstructor` (a
    /// constructor) and returns either a normal completion containing a function object or a throw
    /// completion.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getpromiseresolve
    fn get_promise_resolve(
        promise_constructor: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. Let promiseResolve be ? Get(promiseConstructor, "resolve").
        let promise_resolve = promise_constructor.get("resolve", context)?;

        // 2. If IsCallable(promiseResolve) is false, throw a TypeError exception.
        if let Some(promise_resolve) = promise_resolve.as_callable() {
            // 3. Return promiseResolve.
            Ok(promise_resolve.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("retrieving a non-callable promise resolver")
                .into())
        }
    }
}
