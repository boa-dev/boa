use crate::{
    builtins::string::string_iterator::StringIterator,
    builtins::ArrayIterator,
    object::{GcObject, ObjectInitializer},
    property::{Attribute, DataDescriptor},
    BoaProfiler, Context, Result, Value,
};

#[derive(Debug, Default)]
pub struct IteratorPrototypes {
    iterator_prototype: GcObject,
    array_iterator: GcObject,
    string_iterator: GcObject,
}

impl IteratorPrototypes {
    pub fn init(ctx: &mut Context) -> Self {
        let iterator_prototype = create_iterator_prototype(ctx);
        Self {
            iterator_prototype: iterator_prototype
                .as_gc_object()
                .expect("Iterator prototype is not an object"),
            array_iterator: ArrayIterator::create_prototype(ctx, iterator_prototype.clone())
                .as_gc_object()
                .expect("Array Iterator Prototype is not an object"),
            string_iterator: StringIterator::create_prototype(ctx, iterator_prototype)
                .as_gc_object()
                .expect("String Iterator Prototype is not an object"),
        }
    }

    pub fn array_iterator(&self) -> GcObject {
        self.array_iterator.clone()
    }

    pub fn iterator_prototype(&self) -> GcObject {
        self.iterator_prototype.clone()
    }

    pub fn string_iterator(&self) -> GcObject {
        self.string_iterator.clone()
    }
}

/// CreateIterResultObject( value, done )
///
/// Generates an object supporting the IteratorResult interface.
pub fn create_iter_result_object(ctx: &mut Context, value: Value, done: bool) -> Value {
    let object = Value::new_object(Some(ctx.global_object()));
    // TODO: Fix attributes of value and done
    let value_property = DataDescriptor::new(value, Attribute::all());
    let done_property = DataDescriptor::new(done, Attribute::all());
    object.set_property("value", value_property);
    object.set_property("done", done_property);
    object
}

/// Get an iterator record
pub fn get_iterator(ctx: &mut Context, iterable: Value) -> Result<IteratorRecord> {
    let iterator_function = iterable
        .get_property(ctx.well_known_symbols().iterator_symbol())
        .and_then(|mut p| p.value.take())
        .ok_or_else(|| ctx.construct_type_error("Not an iterable"))?;
    let iterator_object = ctx.call(&iterator_function, &iterable, &[])?;
    let next_function = iterator_object
        .get_property("next")
        .and_then(|mut p| p.value.take())
        .ok_or_else(|| ctx.construct_type_error("Could not find property `next`"))?;
    Ok(IteratorRecord::new(iterator_object, next_function))
}

/// Create the %IteratorPrototype% object
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-%iteratorprototype%-object
fn create_iterator_prototype(ctx: &mut Context) -> Value {
    let _timer = BoaProfiler::global().start_event("Iterator Prototype", "init");

    let symbol_iterator = ctx.well_known_symbols().iterator_symbol();
    let iterator_prototype = ObjectInitializer::new(ctx)
        .function(
            |v, _, _| Ok(v.clone()),
            (symbol_iterator, "[Symbol.iterator]"),
            0,
        )
        .build();
    // TODO: return GcObject
    iterator_prototype.into()
}

#[derive(Debug)]
pub struct IteratorRecord {
    iterator_object: Value,
    next_function: Value,
}

impl IteratorRecord {
    fn new(iterator_object: Value, next_function: Value) -> Self {
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
    pub(crate) fn next(&self, ctx: &mut Context) -> Result<IteratorResult> {
        let next = ctx.call(&self.next_function, &self.iterator_object, &[])?;
        let done = next
            .get_property("done")
            .and_then(|mut p| p.value.take())
            .and_then(|v| v.as_boolean())
            .ok_or_else(|| ctx.construct_type_error("Could not find property `done`"))?;
        let next_result = next
            .get_property("value")
            .and_then(|mut p| p.value.take())
            .unwrap_or_default();
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
