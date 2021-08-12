use crate::{
    builtins::{
        regexp::regexp_string_iterator::RegExpStringIterator,
        string::string_iterator::StringIterator, ArrayIterator, ForInIterator, MapIterator,
        SetIterator,
    },
    object::{GcObject, ObjectInitializer},
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsValue, Result,
};

#[derive(Debug, Default)]
pub struct IteratorPrototypes {
    iterator_prototype: GcObject,
    array_iterator: GcObject,
    set_iterator: GcObject,
    string_iterator: GcObject,
    regexp_string_iterator: GcObject,
    map_iterator: GcObject,
    for_in_iterator: GcObject,
}

impl IteratorPrototypes {
    pub(crate) fn init(context: &mut Context) -> Self {
        let iterator_prototype = create_iterator_prototype(context);
        Self {
            array_iterator: ArrayIterator::create_prototype(
                context,
                iterator_prototype.clone().into(),
            ),
            set_iterator: SetIterator::create_prototype(context, iterator_prototype.clone().into()),
            string_iterator: StringIterator::create_prototype(
                context,
                iterator_prototype.clone().into(),
            ),
            regexp_string_iterator: RegExpStringIterator::create_prototype(
                context,
                iterator_prototype.clone().into(),
            ),
            map_iterator: MapIterator::create_prototype(context, iterator_prototype.clone().into()),
            for_in_iterator: ForInIterator::create_prototype(
                context,
                iterator_prototype.clone().into(),
            ),
            iterator_prototype,
        }
    }

    #[inline]
    pub fn array_iterator(&self) -> GcObject {
        self.array_iterator.clone()
    }

    #[inline]
    pub fn iterator_prototype(&self) -> GcObject {
        self.iterator_prototype.clone()
    }

    #[inline]
    pub fn set_iterator(&self) -> GcObject {
        self.set_iterator.clone()
    }

    #[inline]
    pub fn string_iterator(&self) -> GcObject {
        self.string_iterator.clone()
    }

    #[inline]
    pub fn regexp_string_iterator(&self) -> GcObject {
        self.regexp_string_iterator.clone()
    }

    #[inline]
    pub fn map_iterator(&self) -> GcObject {
        self.map_iterator.clone()
    }

    #[inline]
    pub fn for_in_iterator(&self) -> GcObject {
        self.for_in_iterator.clone()
    }
}

/// CreateIterResultObject( value, done )
///
/// Generates an object supporting the IteratorResult interface.
pub fn create_iter_result_object(context: &mut Context, value: JsValue, done: bool) -> JsValue {
    // 1. Assert: Type(done) is Boolean.
    // 2. Let obj be ! OrdinaryObjectCreate(%Object.prototype%).
    let obj = context.construct_object();

    // 3. Perform ! CreateDataPropertyOrThrow(obj, "value", value).
    obj.create_data_property_or_throw("value", value, context)
        .unwrap();
    // 4. Perform ! CreateDataPropertyOrThrow(obj, "done", done).
    obj.create_data_property_or_throw("done", done, context)
        .unwrap();
    // 5. Return obj.
    obj.into()
}

/// Get an iterator record
pub fn get_iterator(context: &mut Context, iterable: JsValue) -> Result<IteratorRecord> {
    let iterator_function = iterable.get_field(WellKnownSymbols::iterator(), context)?;
    if iterator_function.is_null_or_undefined() {
        return Err(context.construct_type_error("Not an iterable"));
    }
    let iterator_object = context.call(&iterator_function, &iterable, &[])?;
    let next_function = iterator_object.get_field("next", context)?;
    if next_function.is_null_or_undefined() {
        return Err(context.construct_type_error("Could not find property `next`"));
    }
    Ok(IteratorRecord::new(iterator_object, next_function))
}

/// Create the %IteratorPrototype% object
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-%iteratorprototype%-object
fn create_iterator_prototype(context: &mut Context) -> GcObject {
    let _timer = BoaProfiler::global().start_event("Iterator Prototype", "init");

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
pub struct IteratorRecord {
    iterator_object: JsValue,
    next_function: JsValue,
}

impl IteratorRecord {
    pub fn new(iterator_object: JsValue, next_function: JsValue) -> Self {
        Self {
            iterator_object,
            next_function,
        }
    }

    /// Get the next value in the iterator
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratornext
    pub(crate) fn next(&self, context: &mut Context) -> Result<IteratorResult> {
        let next = context.call(&self.next_function, &self.iterator_object, &[])?;
        let done = next.get_field("done", context)?.to_boolean();

        let next_result = next.get_field("value", context)?;
        Ok(IteratorResult::new(next_result, done))
    }

    /// Cleanup the iterator
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    ///  [spec]: https://tc39.es/ecma262/#sec-iteratorclose
    pub(crate) fn close(
        &self,
        completion: Result<JsValue>,
        context: &mut Context,
    ) -> Result<JsValue> {
        let mut inner_result = self.iterator_object.get_field("return", context);

        // 5
        if let Ok(inner_value) = inner_result {
            // b
            if inner_value.is_undefined() {
                return completion;
            }
            // c
            inner_result = context.call(&inner_value, &self.iterator_object, &[]);
        }

        // 6
        let completion = completion?;

        // 7
        let inner_result = inner_result?;

        // 8
        if !inner_result.is_object() {
            return context.throw_type_error("`return` method of iterator didn't return an Object");
        }

        // 9
        Ok(completion)
    }
}

#[derive(Debug)]
pub struct IteratorResult {
    value: JsValue,
    done: bool,
}

impl IteratorResult {
    fn new(value: JsValue, done: bool) -> Self {
        Self { value, done }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn value(self) -> JsValue {
        self.value
    }
}
