use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object, Array, Value},
    object::ObjectData,
    property::{Attribute, DataDescriptor},
    BoaProfiler, Context, Result,
};
use gc::{Finalize, Trace};

#[derive(Debug, Clone, Finalize, Trace)]
pub enum ArrayIterationKind {
    Key,
    Value,
    KeyAndValue,
}

/// The Array Iterator object represents an iteration over an array. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct ArrayIterator {
    array: Value,
    next_index: u32,
    kind: ArrayIterationKind,
}

impl ArrayIterator {
    pub(crate) const NAME: &'static str = "ArrayIterator";

    fn new(array: Value, kind: ArrayIterationKind) -> Self {
        ArrayIterator {
            array,
            kind,
            next_index: 0,
        }
    }

    /// CreateArrayIterator( array, kind )
    ///
    /// Creates a new iterator over the given array.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createarrayiterator
    pub(crate) fn create_array_iterator(
        ctx: &Context,
        array: Value,
        kind: ArrayIterationKind,
    ) -> Result<Value> {
        let array_iterator = Value::new_object(Some(ctx.global_object()));
        array_iterator.set_data(ObjectData::ArrayIterator(Self::new(array, kind)));
        array_iterator
            .as_object()
            .expect("array iterator object")
            .set_prototype_instance(ctx.iterator_prototypes().array_iterator().into());
        Ok(array_iterator)
    }

    /// %ArrayIteratorPrototype%.next( )
    ///
    /// Gets the next result in the array.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%.next
    pub(crate) fn next(this: &Value, _args: &[Value], ctx: &mut Context) -> Result<Value> {
        if let Value::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(array_iterator) = object.as_array_iterator_mut() {
                let index = array_iterator.next_index;
                if array_iterator.array.is_undefined() {
                    return Ok(create_iter_result_object(ctx, Value::undefined(), true));
                }
                let len = array_iterator
                    .array
                    .get_field("length")
                    .as_number()
                    .ok_or_else(|| ctx.construct_type_error("Not an array"))?
                    as u32;
                if array_iterator.next_index >= len {
                    array_iterator.array = Value::undefined();
                    return Ok(create_iter_result_object(ctx, Value::undefined(), true));
                }
                array_iterator.next_index = index + 1;
                match array_iterator.kind {
                    ArrayIterationKind::Key => {
                        Ok(create_iter_result_object(ctx, index.into(), false))
                    }
                    ArrayIterationKind::Value => {
                        let element_value = array_iterator.array.get_field(index);
                        Ok(create_iter_result_object(ctx, element_value, false))
                    }
                    ArrayIterationKind::KeyAndValue => {
                        let element_value = array_iterator.array.get_field(index);
                        let result = Array::constructor(
                            &Value::new_object(Some(ctx.global_object())),
                            &[index.into(), element_value],
                            ctx,
                        )?;
                        Ok(create_iter_result_object(ctx, result, false))
                    }
                }
            } else {
                ctx.throw_type_error("`this` is not an ArrayIterator")
            }
        } else {
            ctx.throw_type_error("`this` is not an ArrayIterator")
        }
    }

    /// Create the %ArrayIteratorPrototype% object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%-object
    pub(crate) fn create_prototype(ctx: &mut Context, iterator_prototype: Value) -> Value {
        let global = ctx.global_object();
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let array_iterator = Value::new_object(Some(global));
        make_builtin_fn(Self::next, "next", &array_iterator, 0, ctx);
        array_iterator
            .as_object()
            .expect("array iterator prototype object")
            .set_prototype_instance(iterator_prototype);

        let to_string_tag = ctx.well_known_symbols().to_string_tag_symbol();
        let to_string_tag_property = DataDescriptor::new("Array Iterator", Attribute::CONFIGURABLE);
        array_iterator.set_property(to_string_tag, to_string_tag_property);
        array_iterator
    }
}
