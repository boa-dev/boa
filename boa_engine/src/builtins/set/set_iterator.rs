//! This module implements the `SetIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-set-iterator-objects

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

/// The Set Iterator object represents an iteration over a set. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-set-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct SetIterator {
    iterated_set: JsValue,
    next_index: usize,
    #[unsafe_ignore_trace]
    iteration_kind: PropertyNameKind,
}

impl IntrinsicObject for SetIterator {
    fn init(intrinsics: &Intrinsics) {
        let _timer = Profiler::global().start_event("SetIterator", "init");

        BuiltInBuilder::with_intrinsic::<Self>(intrinsics)
            .prototype(intrinsics.objects().iterator_prototypes().iterator())
            .static_method(Self::next, "next", 0)
            .static_property(
                JsSymbol::to_string_tag(),
                "Set Iterator",
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().set()
    }
}

impl SetIterator {
    /// Constructs a new `SetIterator`, that will iterate over `set`, starting at index 0
    const fn new(set: JsValue, kind: PropertyNameKind) -> Self {
        Self {
            iterated_set: set,
            next_index: 0,
            iteration_kind: kind,
        }
    }

    /// Abstract operation `CreateSetIterator( set, kind )`
    ///
    /// Creates a new iterator over the given set.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createsetiterator
    pub(crate) fn create_set_iterator(
        set: JsValue,
        kind: PropertyNameKind,
        context: &Context<'_>,
    ) -> JsValue {
        let set_iterator = JsObject::from_proto_and_data(
            context.intrinsics().objects().iterator_prototypes().set(),
            ObjectData::set_iterator(Self::new(set, kind)),
        );
        set_iterator.into()
    }

    /// %SetIteratorPrototype%.next( )
    ///
    /// Advances the iterator and gets the next result in the set.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%setiteratorprototype%.next
    pub(crate) fn next(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let mut set_iterator = this.as_object().map(JsObject::borrow_mut);

        let set_iterator = set_iterator
            .as_mut()
            .and_then(|obj| obj.as_set_iterator_mut())
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not an SetIterator"))?;
        {
            let m = &set_iterator.iterated_set;
            let mut index = set_iterator.next_index;
            let item_kind = &set_iterator.iteration_kind;

            if set_iterator.iterated_set.is_undefined() {
                return Ok(create_iter_result_object(
                    JsValue::undefined(),
                    true,
                    context,
                ));
            }

            let entries = m.as_object().map(JsObject::borrow);
            let entries = entries
                .as_ref()
                .and_then(|obj| obj.as_set())
                .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Set"))?;

            let num_entries = entries.size();
            while index < num_entries {
                let e = entries.get_index(index);
                index += 1;
                set_iterator.next_index = index;
                if let Some(value) = e {
                    match item_kind {
                        PropertyNameKind::Value => {
                            return Ok(create_iter_result_object(value.clone(), false, context));
                        }
                        PropertyNameKind::KeyAndValue => {
                            let result = Array::create_array_from_list(
                                [value.clone(), value.clone()],
                                context,
                            );
                            return Ok(create_iter_result_object(result.into(), false, context));
                        }
                        PropertyNameKind::Key => {
                            panic!("tried to collect only keys of Set")
                        }
                    }
                }
            }
        }

        set_iterator.iterated_set = JsValue::undefined();
        Ok(create_iter_result_object(
            JsValue::undefined(),
            true,
            context,
        ))
    }
}
