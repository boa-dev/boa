use crate::{
    builtins::{
        regexp::regexp_string_iterator::RegExpStringIterator,
        string::string_iterator::StringIterator, ArrayIterator, ForInIterator, MapIterator,
        SetIterator,
    },
    object::{JsObject, ObjectInitializer},
    symbol::WellKnownSymbols,
    Context, JsResult, JsValue,
};
use boa_profiler::Profiler;

#[derive(Debug, Default)]
pub struct IteratorPrototypes {
    /// %IteratorPrototype%
    iterator_prototype: JsObject,
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
                    let _sync_iterator_record =
                        self.get_iterator(context, Some(IteratorHint::Sync), Some(sync_method));
                    // 3. Return ! CreateAsyncFromSyncIterator(syncIteratorRecord).
                    todo!("CreateAsyncFromSyncIterator");
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
        if !iterator.is_object() {
            return context.throw_type_error("the iterator is not an object");
        }

        // 5. Let nextMethod be ? GetV(iterator, "next").
        let next_method = iterator.get_v("next", context)?;

        // 6. Let iteratorRecord be the Record { [[Iterator]]: iterator, [[NextMethod]]: nextMethod, [[Done]]: false }.
        // 7. Return iteratorRecord.
        Ok(IteratorRecord::new(iterator, next_method))
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

#[derive(Debug)]
pub struct IteratorResult {
    object: JsObject,
}

impl IteratorResult {
    /// Get `done` property of iterator result object.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorclose
    #[inline]
    pub fn complete(&self, context: &mut Context) -> JsResult<bool> {
        // 1. Return ToBoolean(? Get(iterResult, "done")).
        Ok(self.object.get("done", context)?.to_boolean())
    }

    /// Get `value` property of iterator result object.
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
/// An Iterator Record is a Record value used to encapsulate an
/// `Iterator` or `AsyncIterator` along with the next method.
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]:https://tc39.es/ecma262/#table-iterator-record-fields
#[derive(Debug)]
pub struct IteratorRecord {
    /// `[[Iterator]]`
    ///
    /// An object that conforms to the Iterator or AsyncIterator interface.
    iterator_object: JsValue,

    /// `[[NextMethod]]`
    ///
    /// The next method of the `[[Iterator]]` object.
    next_function: JsValue,
}

impl IteratorRecord {
    #[inline]
    pub fn new(iterator_object: JsValue, next_function: JsValue) -> Self {
        Self {
            iterator_object,
            next_function,
        }
    }

    #[inline]
    pub(crate) fn iterator_object(&self) -> &JsValue {
        &self.iterator_object
    }

    #[inline]
    pub(crate) fn next_function(&self) -> &JsValue {
        &self.next_function
    }

    /// Get the next value in the iterator
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

        // 1. If value is not present, then
        //     a. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]]).
        // 2. Else,
        //     a. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]], « value »).
        let result = if let Some(value) = value {
            context.call(&self.next_function, &self.iterator_object, &[value])?
        } else {
            context.call(&self.next_function, &self.iterator_object, &[])?
        };

        // 3. If Type(result) is not Object, throw a TypeError exception.
        // 4. Return result.
        if let Some(o) = result.as_object() {
            Ok(IteratorResult { object: o.clone() })
        } else {
            context.throw_type_error("next value should be an object")
        }
    }

    #[inline]
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

    /// Cleanup the iterator
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
        // 3. Let innerResult be GetMethod(iterator, "return").
        let inner_result = self.iterator_object.get_method("return", context);

        // 4. If innerResult.[[Type]] is normal, then
        if let Ok(inner_value) = inner_result {
            // a. Let return be innerResult.[[Value]].
            match inner_value {
                // b. If return is undefined, return Completion(completion).
                None => return completion,
                // c. Set innerResult to Call(return, iterator).
                Some(value) => {
                    let inner_result = value.call(&self.iterator_object, &[], context);

                    // 5. If completion.[[Type]] is throw, return Completion(completion).
                    let completion = completion?;

                    // 6. If innerResult.[[Type]] is throw, return Completion(innerResult).
                    inner_result?;

                    // 7. If Type(innerResult.[[Value]]) is not Object, throw a TypeError exception.
                    // 8. Return Completion(completion).
                    return Ok(completion);
                }
            }
        }

        // 5. If completion.[[Type]] is throw, return Completion(completion).
        let completion = completion?;

        // 6. If innerResult.[[Type]] is throw, return Completion(innerResult).
        inner_result?;

        // 7. If Type(innerResult.[[Value]]) is not Object, throw a TypeError exception.
        // 8. Return Completion(completion).
        Ok(completion)
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

/// A shorthand for a sequence of algorithm steps that use an Iterator Record
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
