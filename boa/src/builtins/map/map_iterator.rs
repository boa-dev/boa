use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object, Array, Value},
    object::{GcObject, ObjectData},
    property::{Attribute, DataDescriptor},
    symbol::WellKnownSymbols,
    BoaProfiler, Context, Result,
};
use gc::{Finalize, Trace};

#[derive(Debug, Clone, Finalize, Trace)]
pub enum MapIterationKind {
    Key,
    Value,
    KeyAndValue,
}

/// The Map Iterator object represents an iteration over a map. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct MapIterator {
    iterated_map: Value,
    map_next_index: usize,
    map_iteration_kind: MapIterationKind,
}

impl MapIterator {
    pub(crate) const NAME: &'static str = "MapIterator";

    fn new(map: Value, kind: MapIterationKind) -> Self {
        MapIterator {
            iterated_map: map,
            map_next_index: 0,
            map_iteration_kind: kind,
        }
    }

    /// Abstract operation CreateMapIterator( map, kind )
    ///
    /// Creates a new iterator over the given map.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://www.ecma-international.org/ecma-262/11.0/index.html#sec-createmapiterator
    pub(crate) fn create_map_iterator(
        context: &Context,
        map: Value,
        kind: MapIterationKind,
    ) -> Value {
        let map_iterator = Value::new_object(context);
        map_iterator.set_data(ObjectData::MapIterator(Self::new(map, kind)));
        map_iterator
            .as_object()
            .expect("map iterator object")
            .set_prototype_instance(context.iterator_prototypes().map_iterator().into());
        map_iterator
    }

    /// %MapIteratorPrototype%.next( )
    ///
    /// Advances the iterator and gets the next result in the map.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%mapiteratorprototype%.next
    pub(crate) fn next(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        if let Value::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(map_iterator) = object.as_map_iterator_mut() {
                let m = &map_iterator.iterated_map;
                let mut index = map_iterator.map_next_index;
                let item_kind = &map_iterator.map_iteration_kind;

                if map_iterator.iterated_map.is_undefined() {
                    return Ok(create_iter_result_object(context, Value::undefined(), true));
                }

                if let Value::Object(ref object) = m {
                    if let Some(entries) = object.borrow().as_map_ref() {
                        let num_entries = entries.len();
                        while index < num_entries {
                            let e = entries.get_index(index);
                            index += 1;
                            map_iterator.map_next_index = index;
                            if let Some((key, value)) = e {
                                match item_kind {
                                    MapIterationKind::Key => {
                                        return Ok(create_iter_result_object(
                                            context,
                                            key.clone(),
                                            false,
                                        ));
                                    }
                                    MapIterationKind::Value => {
                                        return Ok(create_iter_result_object(
                                            context,
                                            value.clone(),
                                            false,
                                        ));
                                    }
                                    MapIterationKind::KeyAndValue => {
                                        let result = Array::construct_array(
                                            &Array::new_array(context),
                                            &[key.clone(), value.clone()],
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
                        return Err(context.construct_type_error("'this' is not a Map"));
                    }
                } else {
                    return Err(context.construct_type_error("'this' is not a Map"));
                }

                map_iterator.iterated_map = Value::undefined();
                Ok(create_iter_result_object(context, Value::undefined(), true))
            } else {
                context.throw_type_error("`this` is not an MapIterator")
            }
        } else {
            context.throw_type_error("`this` is not an MapIterator")
        }
    }

    /// Create the %MapIteratorPrototype% object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%mapiteratorprototype%-object
    pub(crate) fn create_prototype(context: &mut Context, iterator_prototype: Value) -> GcObject {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let mut map_iterator = context.construct_object();
        make_builtin_fn(Self::next, "next", &map_iterator, 0, context);
        map_iterator.set_prototype_instance(iterator_prototype);

        let to_string_tag = WellKnownSymbols::to_string_tag();
        let to_string_tag_property = DataDescriptor::new("Map Iterator", Attribute::CONFIGURABLE);
        map_iterator.insert(to_string_tag, to_string_tag_property);
        map_iterator
    }
}
