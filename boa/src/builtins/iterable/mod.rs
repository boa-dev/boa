use crate::{
    builtins::string::string_iterator::StringIterator,
    builtins::ArrayIterator,
    builtins::ForInIterator,
    builtins::MapIterator,
    object::{GcObject, ObjectInitializer},
    property::{Attribute, DataDescriptor},
    symbol::WellKnownSymbols,
    BoaProfiler, Context, Result, Value,
};

#[derive(Debug, Default)]
pub struct IteratorPrototypes {
    iterator_prototype: GcObject,
    array_iterator: GcObject,
    string_iterator: GcObject,
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
            string_iterator: StringIterator::create_prototype(
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
    pub fn string_iterator(&self) -> GcObject {
        self.string_iterator.clone()
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
pub fn create_iter_result_object(context: &mut Context, value: Value, done: bool) -> Value {
    let object = Value::new_object(context);
    // TODO: Fix attributes of value and done
    let value_property = DataDescriptor::new(value, Attribute::all());
    let done_property = DataDescriptor::new(done, Attribute::all());
    object.set_property("value", value_property);
    object.set_property("done", done_property);
    object
}

/// Get an iterator record
pub fn get_iterator(context: &mut Context, iterable: Value) -> Result<IteratorRecord> {
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
    iterator_object: Value,
    next_function: Value,
}

impl IteratorRecord {
    pub fn new(iterator_object: Value, next_function: Value) -> Self {
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
}

#[derive(Debug)]
pub struct IteratorResult {
    value: Value,
    done: bool,
}

impl IteratorResult {
    fn new(value: Value, done: bool) -> Self {
        Self { value, done }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn value(self) -> Value {
        self.value
    }
}
