//! Boa's implementation of ECMAScript's `IteratorRecord` and iterator prototype objects.

use crate::{
    builtins::{BuiltInBuilder, IntrinsicObject},
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::{JsObject, ObjectData},
    realm::Realm,
    symbol::JsSymbol,
    Context, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

mod async_from_sync_iterator;
pub(crate) use async_from_sync_iterator::AsyncFromSyncIterator;

/// `IfAbruptCloseIterator ( value, iteratorRecord )`
///
/// `IfAbruptCloseIterator` is a shorthand for a sequence of algorithm steps that use an `Iterator`
/// Record.
///
/// More information:
///  - [ECMA reference][spec]
///
///  [spec]: https://tc39.es/ecma262/#sec-ifabruptcloseiterator
macro_rules! if_abrupt_close_iterator {
    ($value:expr, $iterator_record:expr, $context:expr) => {
        match $value {
            // 1. If value is an abrupt completion, return ? IteratorClose(iteratorRecord, value).
            Err(err) => return $iterator_record.close(Err(err), $context),
            // 2. Else if value is a Completion Record, set value to value.
            Ok(value) => value,
        }
    };
}

// Export macro to crate level
pub(crate) use if_abrupt_close_iterator;

/// The built-in iterator prototypes.
#[derive(Debug, Default, Trace, Finalize)]
pub struct IteratorPrototypes {
    /// The `IteratorPrototype` object.
    iterator: JsObject,

    /// The `AsyncIteratorPrototype` object.
    async_iterator: JsObject,

    /// The `AsyncFromSyncIteratorPrototype` prototype object.
    async_from_sync_iterator: JsObject,

    /// The `ArrayIteratorPrototype` prototype object.
    array: JsObject,

    /// The `SetIteratorPrototype` prototype object.
    set: JsObject,

    /// The `StringIteratorPrototype` prototype object.
    string: JsObject,

    /// The `RegExpStringIteratorPrototype` prototype object.
    regexp_string: JsObject,

    /// The `MapIteratorPrototype` prototype object.
    map: JsObject,

    /// The `ForInIteratorPrototype` prototype object.
    for_in: JsObject,

    /// The `%SegmentIteratorPrototype%` prototype object.
    #[cfg(feature = "intl")]
    segment: JsObject,
}

impl IteratorPrototypes {
    /// Returns the `ArrayIteratorPrototype` object.
    #[inline]
    pub fn array(&self) -> JsObject {
        self.array.clone()
    }

    /// Returns the `IteratorPrototype` object.
    #[inline]
    pub fn iterator(&self) -> JsObject {
        self.iterator.clone()
    }

    /// Returns the `AsyncIteratorPrototype` object.
    #[inline]
    pub fn async_iterator(&self) -> JsObject {
        self.async_iterator.clone()
    }

    /// Returns the `AsyncFromSyncIteratorPrototype` object.
    #[inline]
    pub fn async_from_sync_iterator(&self) -> JsObject {
        self.async_from_sync_iterator.clone()
    }

    /// Returns the `SetIteratorPrototype` object.
    #[inline]
    pub fn set(&self) -> JsObject {
        self.set.clone()
    }

    /// Returns the `StringIteratorPrototype` object.
    #[inline]
    pub fn string(&self) -> JsObject {
        self.string.clone()
    }

    /// Returns the `RegExpStringIteratorPrototype` object.
    #[inline]
    pub fn regexp_string(&self) -> JsObject {
        self.regexp_string.clone()
    }

    /// Returns the `MapIteratorPrototype` object.
    #[inline]
    pub fn map(&self) -> JsObject {
        self.map.clone()
    }

    /// Returns the `ForInIteratorPrototype` object.
    #[inline]
    pub fn for_in(&self) -> JsObject {
        self.for_in.clone()
    }

    /// Returns the `%SegmentIteratorPrototype%` object.
    #[inline]
    #[cfg(feature = "intl")]
    pub fn segment(&self) -> JsObject {
        self.segment.clone()
    }
}

/// `%IteratorPrototype%` object
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-%iteratorprototype%-object

pub(crate) struct Iterator;

impl IntrinsicObject for Iterator {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event("Iterator Prototype", "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(
                |v, _, _| Ok(v.clone()),
                (JsSymbol::iterator(), js_string!("[Symbol.iterator]")),
                0,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().iterator()
    }
}

/// `%AsyncIteratorPrototype%` object
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-asynciteratorprototype
pub(crate) struct AsyncIterator;

impl IntrinsicObject for AsyncIterator {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event("AsyncIteratorPrototype", "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(
                |v, _, _| Ok(v.clone()),
                (
                    JsSymbol::async_iterator(),
                    js_string!("[Symbol.asyncIterator]"),
                ),
                0,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().async_iterator()
    }
}

/// `CreateIterResultObject( value, done )`
///
/// Generates an object supporting the `IteratorResult` interface.
pub fn create_iter_result_object(value: JsValue, done: bool, context: &mut Context<'_>) -> JsValue {
    let _timer = Profiler::global().start_event("create_iter_result_object", "init");

    // 1. Assert: Type(done) is Boolean.
    // 2. Let obj be ! OrdinaryObjectCreate(%Object.prototype%).
    // 3. Perform ! CreateDataPropertyOrThrow(obj, "value", value).
    // 4. Perform ! CreateDataPropertyOrThrow(obj, "done", done).
    let obj = context
        .intrinsics()
        .templates()
        .iterator_result()
        .create(ObjectData::ordinary(), vec![value, done.into()]);

    // 5. Return obj.
    obj.into()
}

/// Iterator hint for `GetIterator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorHint {
    /// Hints that the iterator should be sync.
    Sync,

    /// Hints that the iterator should be async.
    Async,
}

impl JsValue {
    /// `GetIterator ( obj [ , hint [ , method ] ] )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getiterator
    pub fn get_iterator(
        &self,
        context: &mut Context<'_>,
        hint: Option<IteratorHint>,
        method: Option<JsObject>,
    ) -> JsResult<IteratorRecord> {
        // 1. If hint is not present, set hint to sync.
        let hint = hint.unwrap_or(IteratorHint::Sync);

        // 2. If method is not present, then
        let method = if method.is_some() {
            method
        } else {
            // a. If hint is async, then
            if hint == IteratorHint::Async {
                // i. Set method to ? GetMethod(obj, @@asyncIterator).
                if let Some(method) = self.get_method(JsSymbol::async_iterator(), context)? {
                    Some(method)
                } else {
                    // ii. If method is undefined, then
                    // 1. Let syncMethod be ? GetMethod(obj, @@iterator).
                    let sync_method = self.get_method(JsSymbol::iterator(), context)?;

                    // 2. Let syncIteratorRecord be ? GetIterator(obj, sync, syncMethod).
                    let sync_iterator_record =
                        self.get_iterator(context, Some(IteratorHint::Sync), sync_method)?;

                    // 3. Return ! CreateAsyncFromSyncIterator(syncIteratorRecord).
                    return Ok(AsyncFromSyncIterator::create(sync_iterator_record, context));
                }
            } else {
                // b. Otherwise, set method to ? GetMethod(obj, @@iterator).
                self.get_method(JsSymbol::iterator(), context)?
            }
        }
        .ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "value with type `{}` is not iterable",
                self.type_of()
            ))
        })?;

        // 3. Let iterator be ? Call(method, obj).
        let iterator = method.call(self, &[], context)?;

        // 4. If Type(iterator) is not Object, throw a TypeError exception.
        let iterator_obj = iterator.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("returned iterator is not an object")
        })?;

        // 5. Let nextMethod be ? GetV(iterator, "next").
        let next_method = iterator.get_v(js_string!("next"), context)?;

        // 6. Let iteratorRecord be the Record { [[Iterator]]: iterator, [[NextMethod]]: nextMethod, [[Done]]: false }.
        // 7. Return iteratorRecord.
        Ok(IteratorRecord::new(iterator_obj.clone(), next_method))
    }
}

/// The result of the iteration process.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct IteratorResult {
    object: JsObject,
}

impl IteratorResult {
    /// Gets a new `IteratorResult` from a value. Returns `Err` if
    /// the value is not a [`JsObject`]
    pub(crate) fn from_value(value: JsValue) -> JsResult<Self> {
        if let JsValue::Object(o) = value {
            Ok(Self { object: o })
        } else {
            Err(JsNativeError::typ()
                .with_message("next value should be an object")
                .into())
        }
    }

    /// Gets the inner object of this `IteratorResult`.
    pub(crate) const fn object(&self) -> &JsObject {
        &self.object
    }

    /// `IteratorComplete ( iterResult )`
    ///
    /// The abstract operation `IteratorComplete` takes argument `iterResult` (an `Object`) and
    /// returns either a normal completion containing a `Boolean` or a throw completion.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorcomplete
    #[inline]
    pub fn complete(&self, context: &mut Context<'_>) -> JsResult<bool> {
        // 1. Return ToBoolean(? Get(iterResult, "done")).
        Ok(self.object.get(js_string!("done"), context)?.to_boolean())
    }

    /// `IteratorValue ( iterResult )`
    ///
    /// The abstract operation `IteratorValue` takes argument `iterResult` (an `Object`) and
    /// returns either a normal completion containing an ECMAScript language value or a throw
    /// completion.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorvalue
    #[inline]
    pub fn value(&self, context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Return ? Get(iterResult, "value").
        self.object.get(js_string!("value"), context)
    }
}

/// Iterator Record
///
/// An Iterator Record is a Record value used to encapsulate an
/// `Iterator` or `AsyncIterator` along with the `next` method.
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator-records
#[derive(Clone, Debug, Finalize, Trace)]
pub struct IteratorRecord {
    /// `[[Iterator]]`
    ///
    /// An object that conforms to the `Iterator` or `AsyncIterator` interface.
    iterator: JsObject,

    /// `[[NextMethod]]`
    ///
    /// The `next` method of the `[[Iterator]]` object.
    next_method: JsValue,

    /// `[[Done]]`
    ///
    /// Whether the iterator has been closed.
    done: bool,

    /// The result of the last call to `next`.
    last_result: IteratorResult,
}

impl IteratorRecord {
    /// Creates a new `IteratorRecord` with the given iterator object, next method and `done` flag.
    #[inline]
    pub fn new(iterator: JsObject, next_method: JsValue) -> Self {
        Self {
            iterator,
            next_method,
            done: false,
            last_result: IteratorResult {
                object: JsObject::with_null_proto(),
            },
        }
    }

    /// Get the `[[Iterator]]` field of the `IteratorRecord`.
    pub(crate) const fn iterator(&self) -> &JsObject {
        &self.iterator
    }

    /// Gets the `[[NextMethod]]` field of the `IteratorRecord`.
    pub(crate) const fn next_method(&self) -> &JsValue {
        &self.next_method
    }

    /// Gets the last result object of the iterator record.
    pub(crate) const fn last_result(&self) -> &IteratorResult {
        &self.last_result
    }

    /// Runs `f`, setting the `done` field of this `IteratorRecord` to `true` if `f` returns
    /// an error.
    fn set_done_on_err<R, F>(&mut self, f: F) -> JsResult<R>
    where
        F: FnOnce(&mut Self) -> JsResult<R>,
    {
        let result = f(self);
        if result.is_err() {
            self.done = true;
        }
        result
    }

    /// Gets the current value of the `IteratorRecord`.
    pub(crate) fn value(&mut self, context: &mut Context<'_>) -> JsResult<JsValue> {
        self.set_done_on_err(|iter| iter.last_result.value(context))
    }

    /// Get the `[[Done]]` field of the `IteratorRecord`.
    pub(crate) const fn done(&self) -> bool {
        self.done
    }

    /// Updates the current result value of this iterator record.
    pub(crate) fn update_result(
        &mut self,
        result: JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        self.set_done_on_err(|iter| {
            // 3. If Type(result) is not Object, throw a TypeError exception.
            // 4. Return result.
            // `IteratorResult::from_value` does this for us.

            // `IteratorStep(iteratorRecord)`
            // https://tc39.es/ecma262/#sec-iteratorstep

            // 1. Let result be ? IteratorNext(iteratorRecord).
            let result = IteratorResult::from_value(result)?;
            // 2. Let done be ? IteratorComplete(result).
            // 3. If done is true, return false.
            iter.done = result.complete(context)?;

            iter.last_result = result;

            Ok(())
        })
    }

    /// `IteratorNext ( iteratorRecord [ , value ] )`
    ///
    /// The abstract operation `IteratorNext` takes argument `iteratorRecord` (an `Iterator`
    /// Record) and optional argument `value` (an ECMAScript language value) and returns either a
    /// normal completion containing an `Object` or a throw completion.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratornext
    pub(crate) fn step_with(
        &mut self,
        value: Option<&JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        let _timer = Profiler::global().start_event("IteratorRecord::step_with", "iterator");

        self.set_done_on_err(|iter| {
            // 1. If value is not present, then
            //     a. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]]).
            // 2. Else,
            //     a. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]], « value »).
            let result = iter.next_method.call(
                &iter.iterator.clone().into(),
                value.map_or(&[], std::slice::from_ref),
                context,
            )?;

            iter.update_result(result, context)?;

            // 4. Return result.
            Ok(iter.done)
        })
    }

    /// `IteratorStep ( iteratorRecord )`
    ///
    /// Updates the `IteratorRecord` and returns `true` if the next result record returned
    /// `done: true`, otherwise returns `false`. This differs slightly from the spec, but also
    /// simplifies some logic around iterators.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorstep
    pub(crate) fn step(&mut self, context: &mut Context<'_>) -> JsResult<bool> {
        self.step_with(None, context)
    }

    /// `IteratorClose ( iteratorRecord, completion )`
    ///
    /// The abstract operation `IteratorClose` takes arguments `iteratorRecord` (an
    /// [Iterator Record][Self]) and `completion` (a `Completion` Record) and returns a
    /// `Completion` Record. It is used to notify an iterator that it should perform any actions it
    /// would normally perform when it has reached its completed state.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    ///  [spec]: https://tc39.es/ecma262/#sec-iteratorclose
    pub(crate) fn close(
        &self,
        completion: JsResult<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let _timer = Profiler::global().start_event("IteratorRecord::close", "iterator");

        // 1. Assert: Type(iteratorRecord.[[Iterator]]) is Object.

        // 2. Let iterator be iteratorRecord.[[Iterator]].
        let iterator = &self.iterator;

        // 3. Let innerResult be Completion(GetMethod(iterator, "return")).
        let inner_result = iterator.get_method(js_string!("return"), context);

        // 4. If innerResult.[[Type]] is normal, then
        let inner_result = match inner_result {
            Ok(inner_result) => {
                // a. Let return be innerResult.[[Value]].
                let r#return = inner_result;

                if let Some(r#return) = r#return {
                    // c. Set innerResult to Completion(Call(return, iterator)).
                    r#return.call(&iterator.clone().into(), &[], context)
                } else {
                    // b. If return is undefined, return ? completion.
                    return completion;
                }
            }
            Err(inner_result) => {
                // 5. If completion.[[Type]] is throw, return ? completion.
                completion?;

                // 6. If innerResult.[[Type]] is throw, return ? innerResult.
                return Err(inner_result);
            }
        };

        // 5. If completion.[[Type]] is throw, return ? completion.
        let completion = completion?;

        // 6. If innerResult.[[Type]] is throw, return ? innerResult.
        let inner_result = inner_result?;

        if inner_result.is_object() {
            // 8. Return ? completion.
            Ok(completion)
        } else {
            // 7. If Type(innerResult.[[Value]]) is not Object, throw a TypeError exception.
            Err(JsNativeError::typ()
                .with_message("inner result was not an object")
                .into())
        }
    }
}

/// `IterableToList ( items [ , method ] )`
///
/// More information:
///  - [ECMA reference][spec]
///
///  [spec]: https://tc39.es/ecma262/#sec-iterabletolist
pub(crate) fn iterable_to_list(
    context: &mut Context<'_>,
    items: &JsValue,
    method: Option<JsObject>,
) -> JsResult<Vec<JsValue>> {
    let _timer = Profiler::global().start_event("iterable_to_list", "iterator");

    // 1. If method is present, then
    // a. Let iteratorRecord be ? GetIterator(items, sync, method).
    // 2. Else,
    // a. Let iteratorRecord be ? GetIterator(items, sync).
    let mut iterator_record = items.get_iterator(context, Some(IteratorHint::Sync), method)?;

    // 3. Let values be a new empty List.
    let mut values = Vec::new();

    // 4. Let next be true.
    // 5. Repeat, while next is not false,
    //     a. Set next to ? IteratorStep(iteratorRecord).
    //     b. If next is not false, then
    //         i. Let nextValue be ? IteratorValue(next).
    //         ii. Append nextValue to the end of the List values.
    while !iterator_record.step(context)? {
        values.push(iterator_record.value(context)?);
    }

    // 6. Return values.
    Ok(values)
}
