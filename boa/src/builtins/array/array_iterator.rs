use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object, Array, JsValue},
    gc::{Finalize, Trace},
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyNameKind},
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult,
};

/// The Array Iterator object represents an iteration over an array. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct ArrayIterator {
    array: JsValue,
    next_index: u32,
    kind: PropertyNameKind,
}

impl ArrayIterator {
    pub(crate) const NAME: &'static str = "ArrayIterator";

    fn new(array: JsValue, kind: PropertyNameKind) -> Self {
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
        array: JsValue,
        kind: PropertyNameKind,
        context: &Context,
    ) -> JsValue {
        let array_iterator = JsValue::new_object(context);
        array_iterator.set_data(ObjectData::array_iterator(Self::new(array, kind)));
        array_iterator
            .as_object()
            .expect("array iterator object")
            .set_prototype_instance(context.iterator_prototypes().array_iterator().into());
        array_iterator
    }

    /// %ArrayIteratorPrototype%.next( )
    ///
    /// Gets the next result in the array.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%.next
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let JsValue::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(array_iterator) = object.as_array_iterator_mut() {
                let index = array_iterator.next_index;
                if array_iterator.array.is_undefined() {
                    return Ok(create_iter_result_object(
                        JsValue::undefined(),
                        true,
                        context,
                    ));
                }
                let len = array_iterator
                    .array
                    .get_field("length", context)?
                    .as_number()
                    .ok_or_else(|| context.construct_type_error("Not an array"))?
                    as u32;
                if array_iterator.next_index >= len {
                    array_iterator.array = JsValue::undefined();
                    return Ok(create_iter_result_object(
                        JsValue::undefined(),
                        true,
                        context,
                    ));
                }
                array_iterator.next_index = index + 1;
                return match array_iterator.kind {
                    PropertyNameKind::Key => {
                        Ok(create_iter_result_object(index.into(), false, context))
                    }
                    PropertyNameKind::Value => {
                        let element_value = array_iterator.array.get_field(index, context)?;
                        Ok(create_iter_result_object(element_value, false, context))
                    }
                    PropertyNameKind::KeyAndValue => {
                        let element_value = array_iterator.array.get_field(index, context)?;
                        let result =
                            Array::create_array_from_list([index.into(), element_value], context);
                        Ok(create_iter_result_object(result.into(), false, context))
                    }
                };
            }
        }
        context.throw_type_error("`this` is not an ArrayIterator")
    }

    /// Create the %ArrayIteratorPrototype% object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%-object
    pub(crate) fn create_prototype(iterator_prototype: JsValue, context: &mut Context) -> JsObject {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let array_iterator = context.construct_object();
        make_builtin_fn(Self::next, "next", &array_iterator, 0, context);
        array_iterator.set_prototype_instance(iterator_prototype);

        let to_string_tag = WellKnownSymbols::to_string_tag();
        let to_string_tag_property = PropertyDescriptor::builder()
            .value("Array Iterator")
            .writable(false)
            .enumerable(false)
            .configurable(true);
        array_iterator.insert(to_string_tag, to_string_tag_property);
        array_iterator
    }
}
