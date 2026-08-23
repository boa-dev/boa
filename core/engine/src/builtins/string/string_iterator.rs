//! This module implements the `StringIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-string-iterator-objects

use crate::{
    Context, JsData, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, IntrinsicObject, iterable::create_iter_result_object},
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};

/// The `StringIterator` object represents an iteration over a string. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-string-iterator-objects
#[derive(Debug, Clone, Finalize, Trace, JsData)]
pub(crate) struct StringIterator {
    string: JsString,
    next_index: usize,
}

impl IntrinsicObject for StringIterator {
    fn init(realm: &Realm, mc: &boa_gc::MutationContext<'static, '_>) {
        BuiltInBuilder::with_intrinsic::<Self>(realm, mc)
            .prototype(realm.intrinsics().constructors().iterator().prototype())
            .static_method(Self::next, js_string!("next"), 0)
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("String Iterator"),
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().string()
    }
}

impl StringIterator {
    /// Create a new `StringIterator`.
    pub(crate) fn create_string_iterator(string: JsString, context: &mut Context) -> JsObject {
        JsObject::from_proto_and_data_with_shared_shape(
            context.gc_collector(),
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .string(),
            Self {
                string,
                next_index: 0,
            },
        )
        .upcast()
    }

    /// `StringIterator.prototype.next( )`
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this
            .as_object()
            .filter(|o| o.is::<Self>())
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a StringIterator"))?;

        let (mut string, position) = {
            let string_iterator = object
                .downcast_ref::<Self>()
                .expect("already checked that it is a StringIterator");
            (string_iterator.string.clone(), string_iterator.next_index)
        };

        if string.is_empty() {
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }
        let len = string.len();
        if position >= len {
            object
                .downcast_mut::<Self>()
                .expect("already checked")
                .string = js_string!();
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }

        let code_point = string.code_point_at(position);
        let next_index = position + code_point.code_unit_count();

        object
            .downcast_mut::<Self>()
            .expect("already checked")
            .next_index = next_index;

        let result_string = crate::builtins::string::String::substring(
            &string.into(),
            &[position.into(), next_index.into()],
            context,
        )?;
        Ok(create_iter_result_object(result_string, false, context))
    }
}
