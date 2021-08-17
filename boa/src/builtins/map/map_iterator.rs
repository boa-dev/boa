use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object, Array, JsValue},
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyNameKind},
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult,
};
use gc::{Finalize, Trace};

use super::{ordered_map::MapLock, Map};
/// The Map Iterator object represents an iteration over a map. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct MapIterator {
    iterated_map: JsValue,
    map_next_index: usize,
    map_iteration_kind: PropertyNameKind,
    lock: MapLock,
}

impl MapIterator {
    pub(crate) const NAME: &'static str = "MapIterator";

    /// Constructs a new `MapIterator`, that will iterate over `map`, starting at index 0
    fn new(map: JsValue, kind: PropertyNameKind, context: &mut Context) -> JsResult<Self> {
        let lock = Map::lock(&map, context)?;
        Ok(MapIterator {
            iterated_map: map,
            map_next_index: 0,
            map_iteration_kind: kind,
            lock,
        })
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
        context: &mut Context,
        map: JsValue,
        kind: PropertyNameKind,
    ) -> JsResult<JsValue> {
        let map_iterator = JsValue::new_object(context);
        map_iterator.set_data(ObjectData::map_iterator(Self::new(map, kind, context)?));
        map_iterator
            .as_object()
            .expect("map iterator object")
            .set_prototype_instance(context.iterator_prototypes().map_iterator().into());
        Ok(map_iterator)
    }

    /// %MapIteratorPrototype%.next( )
    ///
    /// Advances the iterator and gets the next result in the map.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%mapiteratorprototype%.next
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let JsValue::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(map_iterator) = object.as_map_iterator_mut() {
                let m = &map_iterator.iterated_map;
                let mut index = map_iterator.map_next_index;
                let item_kind = &map_iterator.map_iteration_kind;

                if map_iterator.iterated_map.is_undefined() {
                    return Ok(create_iter_result_object(
                        context,
                        JsValue::undefined(),
                        true,
                    ));
                }

                if let JsValue::Object(ref object) = m {
                    if let Some(entries) = object.borrow().as_map_ref() {
                        let num_entries = entries.full_len();
                        while index < num_entries {
                            let e = entries.get_index(index);
                            index += 1;
                            map_iterator.map_next_index = index;
                            if let Some((key, value)) = e {
                                match item_kind {
                                    PropertyNameKind::Key => {
                                        return Ok(create_iter_result_object(
                                            context,
                                            key.clone(),
                                            false,
                                        ));
                                    }
                                    PropertyNameKind::Value => {
                                        return Ok(create_iter_result_object(
                                            context,
                                            value.clone(),
                                            false,
                                        ));
                                    }
                                    PropertyNameKind::KeyAndValue => {
                                        let result = Array::create_array_from_list(
                                            [key.clone(), value.clone()],
                                            context,
                                        );
                                        return Ok(create_iter_result_object(
                                            context,
                                            result.into(),
                                            false,
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

                map_iterator.iterated_map = JsValue::undefined();
                Ok(create_iter_result_object(
                    context,
                    JsValue::undefined(),
                    true,
                ))
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
    pub(crate) fn create_prototype(context: &mut Context, iterator_prototype: JsValue) -> JsObject {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let map_iterator = context.construct_object();
        make_builtin_fn(Self::next, "next", &map_iterator, 0, context);
        map_iterator.set_prototype_instance(iterator_prototype);

        let to_string_tag = WellKnownSymbols::to_string_tag();
        let to_string_tag_property = PropertyDescriptor::builder()
            .value("Map Iterator")
            .writable(false)
            .enumerable(false)
            .configurable(true);
        map_iterator.insert(to_string_tag, to_string_tag_property);
        map_iterator
    }
}
