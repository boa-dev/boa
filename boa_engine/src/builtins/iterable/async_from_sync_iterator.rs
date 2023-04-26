use crate::{
    builtins::{
        iterable::{create_iter_result_object, IteratorRecord, IteratorResult},
        promise::{if_abrupt_reject_promise, PromiseCapability},
        BuiltInBuilder, IntrinsicObject, Promise,
    },
    context::intrinsics::Intrinsics,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject, ObjectData},
    realm::Realm,
    string::utf16,
    Context, JsArgs, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

/// `%AsyncFromSyncIteratorPrototype%` object.
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-properties-of-async-from-sync-iterator-instances
#[derive(Clone, Debug, Finalize, Trace)]
pub struct AsyncFromSyncIterator {
    // The [[SyncIteratorRecord]] internal slot.
    sync_iterator_record: IteratorRecord,
}

impl IntrinsicObject for AsyncFromSyncIterator {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event("AsyncFromSyncIteratorPrototype", "init");

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
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics
            .objects()
            .iterator_prototypes()
            .async_from_sync_iterator()
    }
}

impl AsyncFromSyncIterator {
    /// `CreateAsyncFromSyncIterator ( syncIteratorRecord )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createasyncfromsynciterator
    pub(crate) fn create(
        sync_iterator_record: IteratorRecord,
        context: &mut Context<'_>,
    ) -> IteratorRecord {
        // 1. Let asyncIterator be OrdinaryObjectCreate(%AsyncFromSyncIteratorPrototype%, « [[SyncIteratorRecord]] »).
        // 2. Set asyncIterator.[[SyncIteratorRecord]] to syncIteratorRecord.
        let async_iterator = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .async_from_sync_iterator(),
            ObjectData::async_from_sync_iterator(Self {
                sync_iterator_record,
            }),
        );

        // 3. Let nextMethod be ! Get(asyncIterator, "next").
        let next_method = async_iterator
            .get(utf16!("next"), context)
            .expect("async from sync iterator prototype must have next method");

        // 4. Let iteratorRecord be the Iterator Record { [[Iterator]]: asyncIterator, [[NextMethod]]: nextMethod, [[Done]]: false }.
        // 5. Return iteratorRecord.
        IteratorRecord::new(async_iterator, next_method, false)
    }

    /// `%AsyncFromSyncIteratorPrototype%.next ( [ value ] )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%asyncfromsynciteratorprototype%.next
    fn next(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Assert: O is an Object that has a [[SyncIteratorRecord]] internal slot.
        // 4. Let syncIteratorRecord be O.[[SyncIteratorRecord]].
        let sync_iterator_record = this
            .as_object()
            .expect("async from sync iterator prototype must be object")
            .borrow()
            .as_async_from_sync_iterator()
            .expect("async from sync iterator prototype must be object")
            .sync_iterator_record
            .clone();

        // 3. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let promise_capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail with promise constructor");

        // 5. If value is present, then
        // a. Let result be Completion(IteratorNext(syncIteratorRecord, value)).
        // 6. Else,
        // a. Let result be Completion(IteratorNext(syncIteratorRecord)).
        let result = sync_iterator_record.next(args.get(0), context);

        // 7. IfAbruptRejectPromise(result, promiseCapability).
        if_abrupt_reject_promise!(result, promise_capability, context);

        // 8. Return AsyncFromSyncIteratorContinuation(result, promiseCapability).
        Self::continuation(&result, &promise_capability, context)
    }

    /// `%AsyncFromSyncIteratorPrototype%.return ( [ value ] )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%asyncfromsynciteratorprototype%.return
    fn r#return(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Assert: O is an Object that has a [[SyncIteratorRecord]] internal slot.
        // 4. Let syncIterator be O.[[SyncIteratorRecord]].[[Iterator]].
        let sync_iterator = this
            .as_object()
            .expect("async from sync iterator prototype must be object")
            .borrow()
            .as_async_from_sync_iterator()
            .expect("async from sync iterator prototype must be object")
            .sync_iterator_record
            .iterator()
            .clone();

        // 3. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let promise_capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail with promise constructor");

        // 5. Let return be Completion(GetMethod(syncIterator, "return")).
        let r#return = sync_iterator.get_method(utf16!("return"), context);

        // 6. IfAbruptRejectPromise(return, promiseCapability).
        if_abrupt_reject_promise!(r#return, promise_capability, context);

        let result = match (r#return, args.get(0)) {
            // 7. If return is undefined, then
            (None, _) => {
                // a. Let iterResult be CreateIterResultObject(value, true).
                let iter_result =
                    create_iter_result_object(args.get_or_undefined(0).clone(), true, context);

                // b. Perform ! Call(promiseCapability.[[Resolve]], undefined, « iterResult »).
                promise_capability
                    .resolve()
                    .call(&JsValue::Undefined, &[iter_result], context)
                    .expect("cannot fail according to spec");

                // c. Return promiseCapability.[[Promise]].
                return Ok(promise_capability.promise().clone().into());
            }
            // 8. If value is present, then
            (Some(r#return), Some(value)) => {
                // a. Let result be Completion(Call(return, syncIterator, « value »)).
                r#return.call(&sync_iterator.clone().into(), &[value.clone()], context)
            }
            // 9. Else,
            (Some(r#return), None) => {
                // a. Let result be Completion(Call(return, syncIterator)).
                r#return.call(&sync_iterator.clone().into(), &[], context)
            }
        };

        // 10. IfAbruptRejectPromise(result, promiseCapability).
        if_abrupt_reject_promise!(result, promise_capability, context);

        let Some(result) = result.as_object() else {
            // 11. If Type(result) is not Object, then
            // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « a newly created TypeError object »).
            promise_capability
                .reject()
                .call(
                    &JsValue::Undefined,
                    &[JsNativeError::typ()
                        .with_message("iterator return function returned non-object")
                        .to_opaque(context)
                        .into()],
                    context,
                )
                .expect("cannot fail according to spec");

            // b. Return promiseCapability.[[Promise]].
            return Ok(promise_capability.promise().clone().into());
        };

        // 12. Return AsyncFromSyncIteratorContinuation(result, promiseCapability).
        Self::continuation(
            &IteratorResult {
                object: result.clone(),
            },
            &promise_capability,
            context,
        )
    }

    /// `%AsyncFromSyncIteratorPrototype%.throw ( [ value ] )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%asyncfromsynciteratorprototype%.throw
    fn throw(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Assert: O is an Object that has a [[SyncIteratorRecord]] internal slot.
        // 4. Let syncIterator be O.[[SyncIteratorRecord]].[[Iterator]].
        let sync_iterator = this
            .as_object()
            .expect("async from sync iterator prototype must be object")
            .borrow()
            .as_async_from_sync_iterator()
            .expect("async from sync iterator prototype must be object")
            .sync_iterator_record
            .iterator()
            .clone();

        // 3. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let promise_capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail with promise constructor");

        // 5. Let throw be Completion(GetMethod(syncIterator, "throw")).
        let throw = sync_iterator.get_method(utf16!("throw"), context);

        // 6. IfAbruptRejectPromise(throw, promiseCapability).
        if_abrupt_reject_promise!(throw, promise_capability, context);

        let result = match (throw, args.get(0)) {
            // 7. If throw is undefined, then
            (None, _) => {
                // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « value »).
                promise_capability
                    .reject()
                    .call(
                        &JsValue::Undefined,
                        &[args.get_or_undefined(0).clone()],
                        context,
                    )
                    .expect("cannot fail according to spec");

                // b. Return promiseCapability.[[Promise]].
                return Ok(promise_capability.promise().clone().into());
            }
            // 8. If value is present, then
            (Some(throw), Some(value)) => {
                // a. Let result be Completion(Call(throw, syncIterator, « value »)).
                throw.call(&sync_iterator.clone().into(), &[value.clone()], context)
            }
            // 9. Else,
            (Some(throw), None) => {
                // a. Let result be Completion(Call(throw, syncIterator)).
                throw.call(&sync_iterator.clone().into(), &[], context)
            }
        };

        // 10. IfAbruptRejectPromise(result, promiseCapability).
        if_abrupt_reject_promise!(result, promise_capability, context);

        let Some(result) = result.as_object() else {
            // 11. If Type(result) is not Object, then
            // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « a newly created TypeError object »).
            promise_capability
                .reject()
                .call(
                    &JsValue::Undefined,
                    &[JsNativeError::typ()
                        .with_message("iterator throw function returned non-object")
                        .to_opaque(context)
                        .into()],
                    context,
                )
                .expect("cannot fail according to spec");

            // b. Return promiseCapability.[[Promise]].
            return Ok(promise_capability.promise().clone().into());
        };

        // 12. Return AsyncFromSyncIteratorContinuation(result, promiseCapability).
        Self::continuation(
            &IteratorResult {
                object: result.clone(),
            },
            &promise_capability,
            context,
        )
    }

    /// `AsyncFromSyncIteratorContinuation ( result, promiseCapability )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncfromsynciteratorcontinuation
    fn continuation(
        result: &IteratorResult,
        promise_capability: &PromiseCapability,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. NOTE: Because promiseCapability is derived from the intrinsic %Promise%,
        // the calls to promiseCapability.[[Reject]] entailed by the
        // use IfAbruptRejectPromise below are guaranteed not to throw.

        // 2. Let done be Completion(IteratorComplete(result)).
        let done = result.complete(context);

        // 3. IfAbruptRejectPromise(done, promiseCapability).
        if_abrupt_reject_promise!(done, promise_capability, context);

        // 4. Let value be Completion(IteratorValue(result)).
        let value = result.value(context);

        // 5. IfAbruptRejectPromise(value, promiseCapability).
        if_abrupt_reject_promise!(value, promise_capability, context);

        // 6. Let valueWrapper be Completion(PromiseResolve(%Promise%, value)).
        let value_wrapper = Promise::promise_resolve(
            &context.intrinsics().constructors().promise().constructor(),
            value,
            context,
        );

        // 7. IfAbruptRejectPromise(valueWrapper, promiseCapability).
        if_abrupt_reject_promise!(value_wrapper, promise_capability, context);

        // 8. Let unwrap be a new Abstract Closure with parameters (value)
        // that captures done and performs the following steps when called:
        // 9. Let onFulfilled be CreateBuiltinFunction(unwrap, 1, "", « »).
        let on_fulfilled = FunctionObjectBuilder::new(
            context,
            NativeFunction::from_copy_closure(move |_this, args, context| {
                // a. Return CreateIterResultObject(value, done).
                Ok(create_iter_result_object(
                    args.get_or_undefined(0).clone(),
                    done,
                    context,
                ))
            }),
        )
        .name("")
        .length(1)
        .build();

        // 10. NOTE: onFulfilled is used when processing the "value" property of an
        // IteratorResult object in order to wait for its value if it is a promise and
        // re-package the result in a new "unwrapped" IteratorResult object.

        // 11. Perform PerformPromiseThen(valueWrapper, onFulfilled, undefined, promiseCapability).
        Promise::perform_promise_then(
            &value_wrapper,
            Some(on_fulfilled),
            None,
            Some(promise_capability.clone()),
            context,
        );

        // 12. Return promiseCapability.[[Promise]].
        Ok(promise_capability.promise().clone().into())
    }
}
