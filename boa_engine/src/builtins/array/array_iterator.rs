use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object, Array, JsValue},
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyNameKind},
    symbol::WellKnownSymbols,
    Context, JsResult,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

/// The Array Iterator object represents an iteration over an array. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct ArrayIterator {
    array: JsObject,
    next_index: usize,
    kind: PropertyNameKind,
    done: bool,
}

impl ArrayIterator {
    pub(crate) const NAME: &'static str = "ArrayIterator";

    fn new(array: JsObject, kind: PropertyNameKind) -> Self {
        Self {
            array,
            kind,
            next_index: 0,
            done: false,
        }
    }

    /// `CreateArrayIterator( array, kind )`
    ///
    /// Creates a new iterator over the given array.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createarrayiterator
    pub(crate) fn create_array_iterator(
        array: JsObject,
        kind: PropertyNameKind,
        context: &Context,
    ) -> JsValue {
        let array_iterator = JsObject::from_proto_and_data(
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .array_iterator(),
            ObjectData::array_iterator(Self::new(array, kind)),
        );
        array_iterator.into()
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
        let mut array_iterator = this.as_object().map(JsObject::borrow_mut);
        let array_iterator = array_iterator
            .as_mut()
            .and_then(|obj| obj.as_array_iterator_mut())
            .ok_or_else(|| context.construct_type_error("`this` is not an ArrayIterator"))?;
        let index = array_iterator.next_index;
        if array_iterator.done {
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }

        let len = if let Some(f) = array_iterator.array.borrow().as_typed_array() {
            if f.is_detached() {
                return context.throw_type_error(
                    "Cannot get value from typed array that has a detached array buffer",
                );
            }

            f.array_length()
        } else {
            array_iterator.array.length_of_array_like(context)?
        };

        if index >= len {
            array_iterator.done = true;
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }
        array_iterator.next_index = index + 1;
        match array_iterator.kind {
            PropertyNameKind::Key => Ok(create_iter_result_object(index.into(), false, context)),
            PropertyNameKind::Value => {
                let element_value = array_iterator.array.get(index, context)?;
                Ok(create_iter_result_object(element_value, false, context))
            }
            PropertyNameKind::KeyAndValue => {
                let element_value = array_iterator.array.get(index, context)?;
                let result = Array::create_array_from_list([index.into(), element_value], context);
                Ok(create_iter_result_object(result.into(), false, context))
            }
        }
    }

    /// Create the `%ArrayIteratorPrototype%` object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%-object
    pub(crate) fn create_prototype(
        iterator_prototype: JsObject,
        context: &mut Context,
    ) -> JsObject {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let array_iterator =
            JsObject::from_proto_and_data(iterator_prototype, ObjectData::ordinary());
        make_builtin_fn(Self::next, "next", &array_iterator, 0, context);

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
