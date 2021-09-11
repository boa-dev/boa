use crate::{
    builtins::function::make_builtin_fn,
    builtins::iterable::create_iter_result_object,
    builtins::Array,
    builtins::JsValue,
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyNameKind},
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult,
};
use gc::{Finalize, Trace};

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
    iteration_kind: PropertyNameKind,
}

impl SetIterator {
    pub(crate) const NAME: &'static str = "SetIterator";

    /// Constructs a new `SetIterator`, that will iterate over `set`, starting at index 0
    fn new(set: JsValue, kind: PropertyNameKind) -> Self {
        SetIterator {
            iterated_set: set,
            next_index: 0,
            iteration_kind: kind,
        }
    }

    /// Abstract operation CreateSetIterator( set, kind )
    ///
    /// Creates a new iterator over the given set.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://www.ecma-international.org/ecma-262/11.0/index.html#sec-createsetiterator
    pub(crate) fn create_set_iterator(
        set: JsValue,
        kind: PropertyNameKind,
        context: &Context,
    ) -> JsValue {
        let set_iterator = JsValue::new_object(context);
        set_iterator.set_data(ObjectData::set_iterator(Self::new(set, kind)));
        set_iterator
            .as_object()
            .expect("set iterator object")
            .set_prototype_instance(context.iterator_prototypes().set_iterator().into());
        set_iterator
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
        if let JsValue::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(set_iterator) = object.as_set_iterator_mut() {
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

                if let JsValue::Object(ref object) = m {
                    if let Some(entries) = object.borrow().as_set_ref() {
                        let num_entries = entries.size();
                        while index < num_entries {
                            let e = entries.get_index(index);
                            index += 1;
                            set_iterator.next_index = index;
                            if let Some(value) = e {
                                match item_kind {
                                    PropertyNameKind::Value => {
                                        return Ok(create_iter_result_object(
                                            value.clone(),
                                            false,
                                            context,
                                        ));
                                    }
                                    PropertyNameKind::KeyAndValue => {
                                        let result = Array::create_array_from_list(
                                            [value.clone(), value.clone()],
                                            context,
                                        );
                                        return Ok(create_iter_result_object(
                                            result.into(),
                                            false,
                                            context,
                                        ));
                                    }
                                    PropertyNameKind::Key => {
                                        panic!("tried to collect only keys of Set")
                                    }
                                }
                            }
                        }
                    } else {
                        return Err(context.construct_type_error("'this' is not a Set"));
                    }
                } else {
                    return Err(context.construct_type_error("'this' is not a Set"));
                }

                set_iterator.iterated_set = JsValue::undefined();
                Ok(create_iter_result_object(
                    JsValue::undefined(),
                    true,
                    context,
                ))
            } else {
                context.throw_type_error("`this` is not an SetIterator")
            }
        } else {
            context.throw_type_error("`this` is not an SetIterator")
        }
    }

    /// Create the %SetIteratorPrototype% object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%setiteratorprototype%-object
    pub(crate) fn create_prototype(iterator_prototype: JsValue, context: &mut Context) -> JsObject {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let set_iterator = context.construct_object();
        make_builtin_fn(Self::next, "next", &set_iterator, 0, context);
        set_iterator.set_prototype_instance(iterator_prototype);

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
