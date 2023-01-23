//! This module implements the `ArrayIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects

use crate::{
    builtins::{
        iterable::create_iter_result_object, Array, BuiltInBuilder, IntrinsicObject, JsValue,
    },
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    object::{JsObject, ObjectData},
    property::{Attribute, PropertyNameKind},
    symbol::JsSymbol,
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
    next_index: u64,
    #[unsafe_ignore_trace]
    kind: PropertyNameKind,
    done: bool,
}

impl IntrinsicObject for ArrayIterator {
    fn init(intrinsics: &Intrinsics) {
        let _timer = Profiler::global().start_event("ArrayIterator", "init");

        BuiltInBuilder::with_intrinsic::<Self>(intrinsics)
            .prototype(intrinsics.objects().iterator_prototypes().iterator())
            .static_method(Self::next, "next", 0)
            .static_property(
                JsSymbol::to_string_tag(),
                "Array Iterator",
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().array()
    }
}

impl ArrayIterator {
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
        context: &Context<'_>,
    ) -> JsValue {
        let array_iterator = JsObject::from_proto_and_data(
            context.intrinsics().objects().iterator_prototypes().array(),
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
    pub(crate) fn next(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let mut array_iterator = this.as_object().map(JsObject::borrow_mut);
        let array_iterator = array_iterator
            .as_mut()
            .and_then(|obj| obj.as_array_iterator_mut())
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not an ArrayIterator"))?;
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
                return Err(JsNativeError::typ()
                    .with_message(
                        "Cannot get value from typed array that has a detached array buffer",
                    )
                    .into());
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
}
