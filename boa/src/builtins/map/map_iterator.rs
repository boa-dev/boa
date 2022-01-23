use super::ordered_map::MapLock;
use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object, Array, JsValue},
    gc::{Finalize, Trace},
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyNameKind},
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult,
};

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
                let map_iterator = JsObject::from_proto_and_data(
                    context.iterator_prototypes().map_iterator(),
                    ObjectData::map_iterator(iter),
                );
                return Ok(map_iterator.into());
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
        let mut map_iterator = this.as_object().map(|obj| obj.borrow_mut());
        let map_iterator = map_iterator
            .as_mut()
            .and_then(|obj| obj.as_map_iterator_mut())
            .ok_or_else(|| context.construct_type_error("`this` is not a MapIterator"))?;

        let item_kind = map_iterator.map_iteration_kind;

        if let Some(obj) = map_iterator.iterated_map.take() {
            let e = {
                let map = obj.borrow();
                let entries = map.as_map_ref().expect("iterator should only iterate maps");
                let len = entries.full_len();
                loop {
                    let element = entries
                        .get_index(map_iterator.map_next_index)
                        .map(|(v, k)| (v.clone(), k.clone()));
                    map_iterator.map_next_index += 1;
                    if element.is_some() || map_iterator.map_next_index >= len {
                        break element;
                    }
                }
            };
            if let Some((key, value)) = e {
                let item = match item_kind {
                    PropertyNameKind::Key => Ok(create_iter_result_object(key, false, context)),
                    PropertyNameKind::Value => Ok(create_iter_result_object(value, false, context)),
                    PropertyNameKind::KeyAndValue => {
                        let result = Array::create_array_from_list([key, value], context);
                        Ok(create_iter_result_object(result.into(), false, context))
                    }
                };
                map_iterator.iterated_map = Some(obj);
                return item;
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
    pub(crate) fn create_prototype(
        iterator_prototype: JsObject,
        context: &mut Context,
    ) -> JsObject {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let map_iterator =
            JsObject::from_proto_and_data(iterator_prototype, ObjectData::ordinary());
        make_builtin_fn(Self::next, "next", &map_iterator, 0, context);

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
