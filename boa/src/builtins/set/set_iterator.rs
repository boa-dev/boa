use crate::{
    builtins::function::make_builtin_fn,
    builtins::iterable::create_iter_result_object,
    builtins::Array,
    builtins::Value,
    object::{GcObject, ObjectData},
    property::{Attribute, DataDescriptor},
    BoaProfiler, Context, Result,
};
use gc::{Finalize, Trace};

#[derive(Debug, Clone, Finalize, Trace)]
pub enum SetIterationKind {
    Value,
    KeyAndValue,
}

/// The Set Iterator object represents an iteration over a set. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-set-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct SetIterator {
    iterated_set: Value,
    next_index: usize,
    iteration_kind: SetIterationKind,
}

impl SetIterator {
    pub(crate) const NAME: &'static str = "SetIterator";

    /// Constructs a new `SetIterator`, that will iterate over `set`, starting at index 0
    fn new(set: Value, kind: SetIterationKind) -> Self {
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
        context: &Context,
        set: Value,
        kind: SetIterationKind,
    ) -> Value {
        let set_iterator = Value::new_object(context);
        set_iterator.set_data(ObjectData::SetIterator(Self::new(set, kind)));
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
    pub(crate) fn next(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        if let Value::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(set_iterator) = object.as_set_iterator_mut() {
                let m = &set_iterator.iterated_set;
                let mut index = set_iterator.next_index;
                let item_kind = &set_iterator.iteration_kind;

                if set_iterator.iterated_set.is_undefined() {
                    return Ok(create_iter_result_object(context, Value::undefined(), true));
                }

                if let Value::Object(ref object) = m {
                    if let Some(entries) = object.borrow().as_set_ref() {
                        let num_entries = entries.size();
                        while index < num_entries {
                            let e = entries.get_index(index);
                            index += 1;
                            set_iterator.next_index = index;
                            if let Some(value) = e {
                                match item_kind {
                                    SetIterationKind::Value => {
                                        return Ok(create_iter_result_object(
                                            context,
                                            value.clone(),
                                            false,
                                        ));
                                    }
                                    SetIterationKind::KeyAndValue => {
                                        let result = Array::construct_array(
                                            &Array::new_array(context),
                                            &[value.clone(), value.clone()],
                                            context,
                                        )?;
                                        return Ok(create_iter_result_object(
                                            context, result, false,
                                        ));
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

                set_iterator.iterated_set = Value::undefined();
                Ok(create_iter_result_object(context, Value::undefined(), true))
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
    pub(crate) fn create_prototype(context: &mut Context, iterator_prototype: Value) -> GcObject {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let mut set_iterator = context.construct_object();
        make_builtin_fn(Self::next, "next", &set_iterator, 0, context);
        set_iterator.set_prototype_instance(iterator_prototype);

        let to_string_tag = context.well_known_symbols().to_string_tag_symbol();
        let to_string_tag_property = DataDescriptor::new("Set Iterator", Attribute::CONFIGURABLE);
        set_iterator.insert(to_string_tag, to_string_tag_property);
        set_iterator
    }
}
