//! This module implements the `ArrayIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects

use crate::{
    Context, JsData, JsResult,
    builtins::{
        Array, BuiltInBuilder, IntrinsicObject, JsValue, iterable::create_iter_result_object,
        typed_array::TypedArray,
    },
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::{Attribute, PropertyNameKind},
    realm::Realm,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};

/// The Array Iterator object represents an iteration over an array. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects
#[derive(Debug, Clone, Finalize, Trace, JsData)]
pub(crate) struct ArrayIterator {
    array: JsObject,
    next_index: u64,
    #[unsafe_ignore_trace]
    kind: PropertyNameKind,
    done: bool,
}

impl IntrinsicObject for ArrayIterator {
    fn init(realm: &Realm, mc: &boa_gc::MutationContext<'static, '_>) {
        BuiltInBuilder::with_intrinsic::<Self>(realm, mc)
            .prototype(realm.intrinsics().constructors().iterator().prototype())
            .static_method(Self::next, js_string!("next"), 0)
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("Array Iterator"),
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
        context: &Context,
    ) -> JsValue {
        let array_iterator = JsObject::from_proto_and_data_with_shared_shape(
            context.gc_collector(),
            context.root_shape(),
            context.intrinsics().objects().iterator_prototypes().array(),
            Self::new(array, kind),
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
        let object = this
            .as_object()
            .filter(|o| o.is::<Self>())
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not an ArrayIterator"))?;

        // Extract needed fields into a scoped block so the RefMut borrow is dropped
        // before any context call. Holding a RefMut<'_, T> across context operations
        // is a use-after-free: GC can collect the backing object while the guard is live.
        let (index, done, array, kind) = {
            let array_iterator = object
                .downcast_ref::<Self>()
                .expect("already checked that it is an ArrayIterator");
            (
                array_iterator.next_index,
                array_iterator.done,
                array_iterator.array.clone(),
                array_iterator.kind,
            )
        };
        // RefMut dropped here — safe to use context below.

        if done {
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }

        let len = if let Some(f) = array.downcast_ref::<TypedArray>() {
            let buf = f.viewed_array_buffer().as_buffer();
            let Some(buf) = buf
                .bytes(std::sync::atomic::Ordering::SeqCst)
                .filter(|buf| !f.is_out_of_bounds(buf.len()))
            else {
                return Err(JsNativeError::typ()
                    .with_message("Cannot get value from out of bounds typed array")
                    .into());
            };

            f.array_length(buf.len())
        } else {
            array.length_of_array_like(context)?
        };

        if index >= len {
            object.downcast_mut::<Self>().expect("already checked").done = true;
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }

        // Write back the incremented index (no borrow held during context calls above).
        object
            .downcast_mut::<Self>()
            .expect("already checked")
            .next_index = index + 1;

        match kind {
            PropertyNameKind::Key => Ok(create_iter_result_object(index.into(), false, context)),
            PropertyNameKind::Value => {
                let element_value = array.get(index, context)?;
                Ok(create_iter_result_object(element_value, false, context))
            }
            PropertyNameKind::KeyAndValue => {
                let element_value = array.get(index, context)?;
                let result = Array::create_array_from_list([index.into(), element_value], context);
                Ok(create_iter_result_object(result.into(), false, context))
            }
        }
    }
}
