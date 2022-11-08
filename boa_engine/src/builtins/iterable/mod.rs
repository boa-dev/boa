mod async_from_sync_iterator;

use crate::{
    builtins::{
        regexp::regexp_string_iterator::RegExpStringIterator,
        string::string_iterator::StringIterator, ArrayIterator, ForInIterator, MapIterator,
        SetIterator,
    },
    error::JsNativeError,
    object::{JsObject, ObjectInitializer},
    symbol::WellKnownSymbols,
    Context, JsResult, JsValue,
};
use async_from_sync_iterator::create_async_from_sync_iterator_prototype;
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

pub(crate) use async_from_sync_iterator::AsyncFromSyncIterator;

#[derive(Debug, Default)]
pub struct IteratorPrototypes {
    /// %IteratorPrototype%
    iterator_prototype: JsObject,
    /// %AsyncIteratorPrototype%
    async_iterator_prototype: JsObject,
    /// %AsyncFromSyncIteratorPrototype%
    async_from_sync_iterator_prototype: JsObject,
    /// %MapIteratorPrototype%
    array_iterator: JsObject,
    /// %SetIteratorPrototype%
    set_iterator: JsObject,
    /// %StringIteratorPrototype%
    string_iterator: JsObject,
    /// %RegExpStringIteratorPrototype%
    regexp_string_iterator: JsObject,
    /// %MapIteratorPrototype%
    map_iterator: JsObject,
    /// %ForInIteratorPrototype%
    for_in_iterator: JsObject,
}

impl IteratorPrototypes {
    pub(crate) fn init(context: &mut Context) -> Self {
        let _timer = Profiler::global().start_event("IteratorPrototypes::init", "init");

        let iterator_prototype = create_iterator_prototype(context);
        let async_iterator_prototype = create_async_iterator_prototype(context);
        let async_from_sync_iterator_prototype = create_async_from_sync_iterator_prototype(context);
        Self {
            array_iterator: ArrayIterator::create_prototype(iterator_prototype.clone(), context),
            set_iterator: SetIterator::create_prototype(iterator_prototype.clone(), context),
            string_iterator: StringIterator::create_prototype(iterator_prototype.clone(), context),
            regexp_string_iterator: RegExpStringIterator::create_prototype(
                iterator_prototype.clone(),
                context,
            ),
            map_iterator: MapIterator::create_prototype(iterator_prototype.clone(), context),
            for_in_iterator: ForInIterator::create_prototype(iterator_prototype.clone(), context),
            iterator_prototype,
            async_iterator_prototype,
            async_from_sync_iterator_prototype,
        }
    }

    #[inline]
    pub fn array_iterator(&self) -> JsObject {
        self.array_iterator.clone()
    }

    #[inline]
    pub fn iterator_prototype(&self) -> JsObject {
        self.iterator_prototype.clone()
    }

    #[inline]
    pub fn async_iterator_prototype(&self) -> JsObject {
        self.async_iterator_prototype.clone()
    }

    #[inline]
    pub fn async_from_sync_iterator_prototype(&self) -> JsObject {
        self.async_from_sync_iterator_prototype.clone()
    }

    #[inline]
    pub fn set_iterator(&self) -> JsObject {
        self.set_iterator.clone()
    }

    #[inline]
    pub fn string_iterator(&self) -> JsObject {
        self.string_iterator.clone()
    }

    #[inline]
    pub fn regexp_string_iterator(&self) -> JsObject {
        self.regexp_string_iterator.clone()
    }

    #[inline]
    pub fn map_iterator(&self) -> JsObject {
        self.map_iterator.clone()
    }

    #[inline]
    pub fn for_in_iterator(&self) -> JsObject {
        self.for_in_iterator.clone()
    }
}

/// `CreateIterResultObject( value, done )`
///
/// Generates an object supporting the `IteratorResult` interface.
#[inline]
pub fn create_iter_result_object(value: JsValue, done: bool, context: &mut Context) -> JsValue {
    let _timer = Profiler::global().start_event("create_iter_result_object", "init");

    // 1. Assert: Type(done) is Boolean.
    // 2. Let obj be ! OrdinaryObjectCreate(%Object.prototype%).
    let obj = context.construct_object();

    // 3. Perform ! CreateDataPropertyOrThrow(obj, "value", value).
    obj.create_data_property_or_throw("value", value, context)
        .expect("this CreateDataPropertyOrThrow call must not fail");
    // 4. Perform ! CreateDataPropertyOrThrow(obj, "done", done).
    obj.create_data_property_or_throw("done", done, context)
        .expect("this CreateDataPropertyOrThrow call must not fail");
    // 5. Return obj.
    obj.into()
}

/// Iterator hint for `GetIterator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorHint {
    Sync,
    Async,
}

impl JsValue {
    /// `GetIterator ( obj [ , hint [ , method ] ] )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getiterator
    #[inline]
    pub fn get_iterator(
        &self,
        context: &mut Context,
        hint: Option<IteratorHint>,
        method: Option<Self>,
    ) -> JsResult<IteratorRecord> {
        // 1. If hint is not present, set hint to sync.
        let hint = hint.unwrap_or(IteratorHint::Sync);

        // 2. If method is not present, then
        let method = if let Some(method) = method {
            method
        } else {
            // a. If hint is async, then
            if hint == IteratorHint::Async {
                // i. Set method to ? GetMethod(obj, @@asyncIterator).
                if let Some(method) =
                    self.get_method(WellKnownSymbols::async_iterator(), context)?
                {
                    method.into()
                } else {
                    // ii. If method is undefined, then
                    // 1. Let syncMethod be ? GetMethod(obj, @@iterator).
                    let sync_method = self
                        .get_method(WellKnownSymbols::iterator(), context)?
                        .map_or(Self::Undefined, Self::from);

                    // 2. Let syncIteratorRecord be ? GetIterator(obj, sync, syncMethod).
                    let sync_iterator_record =
                        self.get_iterator(context, Some(IteratorHint::Sync), Some(sync_method))?;

                    // 3. Return ! CreateAsyncFromSyncIterator(syncIteratorRecord).
                    return Ok(AsyncFromSyncIterator::create(sync_iterator_record, context));
                }
            } else {
                // b. Otherwise, set method to ? GetMethod(obj, @@iterator).
                self.get_method(WellKnownSymbols::iterator(), context)?
                    .map_or(Self::Undefined, Self::from)
            }
        };

        // 3. Let iterator be ? Call(method, obj).
        let iterator = context.call(&method, self, &[])?;

        // 4. If Type(iterator) is not Object, throw a TypeError exception.
        let iterator_obj = iterator
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("the iterator is not an object"))?;

        // 5. Let nextMethod be ? GetV(iterator, "next").
        let next_method = iterator.get_v("next", context)?;

        // 6. Let iteratorRecord be the Record { [[Iterator]]: iterator, [[NextMethod]]: nextMethod, [[Done]]: false }.
        // 7. Return iteratorRecord.
        Ok(IteratorRecord::new(
            iterator_obj.clone(),
            next_method,
            false,
        ))
    }
}

/// Create the `%IteratorPrototype%` object
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-%iteratorprototype%-object
#[inline]
fn create_iterator_prototype(context: &mut Context) -> JsObject {
    let _timer = Profiler::global().start_event("Iterator Prototype", "init");

    let symbol_iterator = WellKnownSymbols::iterator();
    let iterator_prototype = ObjectInitializer::new(context)
        .function(
            |v, _, _| Ok(v.clone()),
            (symbol_iterator, "[Symbol.iterator]"),
            0,
        )
        .build();
    iterator_prototype
}

/// The result of the iteration process.
#[derive(Debug)]
pub struct IteratorResult {
    object: JsObject,
}

impl IteratorResult {
    /// Create a new `IteratorResult`.
    pub(crate) fn new(object: JsObject) -> Self {
        Self { object }
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
    pub fn complete(&self, context: &mut Context) -> JsResult<bool> {
        // 1. Return ToBoolean(? Get(iterResult, "done")).
        Ok(self.object.get("done", context)?.to_boolean())
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
    pub fn value(&self, context: &mut Context) -> JsResult<JsValue> {
        // 1. Return ? Get(iterResult, "value").
        self.object.get("value", context)
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
}

impl IteratorRecord {
    /// Creates a new `IteratorRecord` with the given iterator object, next method and `done` flag.
    #[inline]
    pub fn new(iterator: JsObject, next_method: JsValue, done: bool) -> Self {
        Self {
            iterator,
            next_method,
            done,
        }
    }

    /// Get the `[[Iterator]]` field of the `IteratorRecord`.
    #[inline]
    pub(crate) fn iterator(&self) -> &JsObject {
        &self.iterator
    }

    /// Get the `[[NextMethod]]` field of the `IteratorRecord`.
    #[inline]
    pub(crate) fn next_method(&self) -> &JsValue {
        &self.next_method
    }

    /// Get the `[[Done]]` field of the `IteratorRecord`.
    #[inline]
    pub(crate) fn done(&self) -> bool {
        self.done
    }

    /// Sets the `[[Done]]` field of the `IteratorRecord`.
    #[inline]
    pub(crate) fn set_done(&mut self, done: bool) {
        self.done = done;
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
    #[inline]
    pub(crate) fn next(
        &self,
        value: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<IteratorResult> {
        let _timer = Profiler::global().start_event("IteratorRecord::next", "iterator");

        // Note: We check if iteratorRecord.[[NextMethod]] is callable here.
        // This check would happen in `Call` according to the spec, but we do not implement call for `JsValue`.
        let next_method = self.next_method.as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("iterable next method not a function")
        })?;

        let result = if let Some(value) = value {
            // 2. Else,
            //     a. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]], « value »).
            next_method.call(&self.iterator.clone().into(), &[value], context)?
        } else {
            // 1. If value is not present, then
            //     a. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]]).
            next_method.call(&self.iterator.clone().into(), &[], context)?
        };

        // 3. If Type(result) is not Object, throw a TypeError exception.
        // 4. Return result.
        result
            .as_object()
            .map(|o| IteratorResult { object: o.clone() })
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("next value should be an object")
                    .into()
            })
    }

    /// `IteratorStep ( iteratorRecord )`
    ///
    /// The abstract operation `IteratorStep` takes argument `iteratorRecord` (an `Iterator`
    /// Record) and returns either a normal completion containing either an `Object` or `false`, or
    /// a throw completion. It requests the next value from `iteratorRecord.[[Iterator]]` by
    /// calling `iteratorRecord.[[NextMethod]]` and returns either `false` indicating that the
    /// iterator has reached its end or the `IteratorResult` object if a next value is available.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorstep
    pub(crate) fn step(&self, context: &mut Context) -> JsResult<Option<IteratorResult>> {
        let _timer = Profiler::global().start_event("IteratorRecord::step", "iterator");

        // 1. Let result be ? IteratorNext(iteratorRecord).
        let result = self.next(None, context)?;

        // 2. Let done be ? IteratorComplete(result).
        let done = result.complete(context)?;

        // 3. If done is true, return false.
        if done {
            return Ok(None);
        }

        // 4. Return result.
        Ok(Some(result))
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
    #[inline]
    pub(crate) fn close(
        &self,
        completion: JsResult<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let _timer = Profiler::global().start_event("IteratorRecord::close", "iterator");

        // 1. Assert: Type(iteratorRecord.[[Iterator]]) is Object.

        // 2. Let iterator be iteratorRecord.[[Iterator]].
        let iterator = &self.iterator;

        // 3. Let innerResult be Completion(GetMethod(iterator, "return")).
        let inner_result = iterator.get_method("return", context);

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
    context: &mut Context,
    items: &JsValue,
    method: Option<JsValue>,
) -> JsResult<Vec<JsValue>> {
    let _timer = Profiler::global().start_event("iterable_to_list", "iterator");

    // 1. If method is present, then
    let iterator_record = if let Some(method) = method {
        // a. Let iteratorRecord be ? GetIterator(items, sync, method).
        items.get_iterator(context, Some(IteratorHint::Sync), Some(method))?
    } else {
        // 2. Else,

        // a. Let iteratorRecord be ? GetIterator(items, sync).
        items.get_iterator(context, Some(IteratorHint::Sync), None)?
    };

    // 3. Let values be a new empty List.
    let mut values = Vec::new();

    // 4. Let next be true.
    // 5. Repeat, while next is not false,
    //     a. Set next to ? IteratorStep(iteratorRecord).
    //     b. If next is not false, then
    //         i. Let nextValue be ? IteratorValue(next).
    //         ii. Append nextValue to the end of the List values.
    while let Some(next) = iterator_record.step(context)? {
        let next_value = next.value(context)?;
        values.push(next_value);
    }

    // 6. Return values.
    Ok(values)
}

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

/// Create the `%AsyncIteratorPrototype%` object
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-asynciteratorprototype
#[inline]
fn create_async_iterator_prototype(context: &mut Context) -> JsObject {
    let _timer = Profiler::global().start_event("AsyncIteratorPrototype", "init");

    let symbol_iterator = WellKnownSymbols::async_iterator();
    let iterator_prototype = ObjectInitializer::new(context)
        .function(
            |v, _, _| Ok(v.clone()),
            (symbol_iterator, "[Symbol.asyncIterator]"),
            0,
        )
        .build();
    iterator_prototype
}
