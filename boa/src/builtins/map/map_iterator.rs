use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object, Array, JsValue},
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyNameKind},
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult,
};
use gc::{Finalize, Trace};

use super::ordered_map::MapLock;
/// The Map Iterator object represents an iteration over a map. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct MapIterator {
    iterated_map: Option<JsObject>,
    map_next_index: usize,
    map_iteration_kind: PropertyNameKind,
    lock: MapLock,
}

impl MapIterator {
    pub(crate) const NAME: &'static str = "MapIterator";

    /// Abstract operation CreateMapIterator( map, kind )
    ///
    /// Creates a new iterator over the given map.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://www.ecma-international.org/ecma-262/11.0/index.html#sec-createmapiterator
    pub(crate) fn create_map_iterator(
        map: &JsValue,
        kind: PropertyNameKind,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let Some(map_obj) = map.as_object() {
            if let Some(map) = map_obj.borrow_mut().as_map_mut() {
                let lock = map.lock(map_obj.clone());
                let iter = MapIterator {
                    iterated_map: Some(map_obj.clone()),
                    map_next_index: 0,
                    map_iteration_kind: kind,
                    lock,
                };
                let map_iterator = JsValue::new_object(context);
                map_iterator.set_data(ObjectData::map_iterator(iter));
                map_iterator
                    .as_object()
                    .expect("map iterator object")
                    .set_prototype_instance(context.iterator_prototypes().map_iterator().into());
                return Ok(map_iterator);
            }
        }
        context.throw_type_error("`this` is not a Map")
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
        let iterator_object = match this {
            JsValue::Object(obj) if obj.borrow().is_map_iterator() => obj,
            _ => return context.throw_type_error("`this` is not a MapIterator"),
        };

        let mut iterator_object = iterator_object.borrow_mut();

        let map_iterator = iterator_object
            .as_map_iterator_mut()
            .expect("checked that obj was a map iterator");

        let mut index = map_iterator.map_next_index;
        let item_kind = map_iterator.map_iteration_kind;

        if let Some(obj) = map_iterator.iterated_map.take() {
            let map = obj.borrow();
            let entries = map.as_map_ref().expect("iterator should only iterate maps");
            let num_entries = entries.full_len();
            while index < num_entries {
                let e = entries.get_index(index);
                index += 1;
                map_iterator.map_next_index = index;
                if let Some((key, value)) = e {
                    let item = match item_kind {
                        PropertyNameKind::Key => {
                            Ok(create_iter_result_object(key.clone(), false, context))
                        }
                        PropertyNameKind::Value => {
                            Ok(create_iter_result_object(value.clone(), false, context))
                        }
                        PropertyNameKind::KeyAndValue => {
                            let result = Array::create_array_from_list(
                                [key.clone(), value.clone()],
                                context,
                            );
                            Ok(create_iter_result_object(result.into(), false, context))
                        }
                    };
                    drop(map);
                    map_iterator.iterated_map = Some(obj);
                    return item;
                }
            }
        }

        Ok(create_iter_result_object(
            JsValue::undefined(),
            true,
            context,
        ))
    }

    /// Create the %MapIteratorPrototype% object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%mapiteratorprototype%-object
    pub(crate) fn create_prototype(iterator_prototype: JsValue, context: &mut Context) -> JsObject {
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
