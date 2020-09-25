use crate::builtins::function::{BuiltInFunction, Function, FunctionFlags};
use crate::builtins::ArrayIterator;
use crate::object::{Object, PROTOTYPE};
use crate::BoaProfiler;
use crate::{property::Property, Context, Value};

#[derive(Debug, Default)]
pub struct IteratorPrototypes {
    iterator_prototype: Value,
    array_iterator: Value,
}

impl IteratorPrototypes {
    pub fn init(ctx: &mut Context) -> Self {
        let iterator_prototype = create_iterator_prototype(ctx);
        Self {
            iterator_prototype: iterator_prototype.clone(),
            array_iterator: ArrayIterator::create_prototype(ctx, iterator_prototype),
        }
    }

    pub fn array_iterator(&self) -> Value {
        self.array_iterator.clone()
    }

    pub fn iterator_prototype(&self) -> Value {
        self.iterator_prototype.clone()
    }
}

/// CreateIterResultObject( value, done )
///
/// Generates an object supporting the IteratorResult interface.
pub fn create_iter_result_object(ctx: &mut Context, value: Value, done: bool) -> Value {
    let object = Value::new_object(Some(
        &ctx.realm()
            .environment
            .get_global_object()
            .expect("Could not get global object"),
    ));
    let value_property = Property::default().value(value);
    let done_property = Property::default().value(Value::boolean(done));
    object.set_property("value", value_property);
    object.set_property("done", done_property);
    object
}

/// Get an iterator record
pub fn get_iterator(ctx: &mut Context, iterable: Value) -> Result<IteratorRecord, Value> {
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
    let global = ctx.global_object();
    let _timer = BoaProfiler::global().start_event("Iterator Prototype", "init");

    let iterator_prototype = Value::new_object(Some(global));
    let mut function = Object::function(
        Function::BuiltIn(
            BuiltInFunction(|v, _, _| Ok(v.clone())),
            FunctionFlags::CALLABLE,
        ),
        global.get_field("Function").get_field(PROTOTYPE),
    );
    function.insert_field("length", Value::from(0));
    function.insert_field("name", Value::string("[Symbol.iterator]"));

    let symbol_iterator = ctx.well_known_symbols().iterator_symbol();
    iterator_prototype.set_field(symbol_iterator, Value::from(function));
    iterator_prototype
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
    pub(crate) fn next(&self, ctx: &mut Context) -> Result<IteratorResult, Value> {
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
