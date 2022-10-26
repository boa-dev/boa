use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object, Array, JsValue},
    error::JsNativeError,
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyNameKind},
    symbol::WellKnownSymbols,
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

impl SetIterator {
    pub(crate) const NAME: &'static str = "SetIterator";

    /// Constructs a new `SetIterator`, that will iterate over `set`, starting at index 0
    fn new(set: JsValue, kind: PropertyNameKind) -> Self {
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
        context: &Context,
    ) -> JsValue {
        let set_iterator = JsObject::from_proto_and_data(
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .set_iterator(),
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
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
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
                .and_then(|obj| obj.as_set_ref())
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

    /// Create the `%SetIteratorPrototype%` object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%setiteratorprototype%-object
    pub(crate) fn create_prototype(
        iterator_prototype: JsObject,
        context: &mut Context,
    ) -> JsObject {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let set_iterator =
            JsObject::from_proto_and_data(iterator_prototype, ObjectData::ordinary());
        make_builtin_fn(Self::next, "next", &set_iterator, 0, context);

        let to_string_tag = WellKnownSymbols::to_string_tag();
        let to_string_tag_property = PropertyDescriptor::builder()
            .value("Set Iterator")
            .writable(false)
            .enumerable(false)
            .configurable(true);
        set_iterator.insert(to_string_tag, to_string_tag_property);
        set_iterator
    }
}
