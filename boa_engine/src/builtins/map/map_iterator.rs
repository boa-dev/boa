//! This module implements the `MapIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-map-iterator-objects

use super::ordered_map::MapLock;
use crate::{
    builtins::{
        iterable::create_iter_result_object, Array, BuiltInBuilder, IntrinsicObject, JsValue,
    },
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::{JsObject, ObjectData},
    property::{Attribute, PropertyNameKind},
    realm::Realm,
    symbol::JsSymbol,
    Context, JsResult,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

/// The Map Iterator object represents an iteration over a map. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-map-iterator-objects
#[derive(Debug, Finalize, Trace)]
pub struct MapIterator {
    iterated_map: Option<JsObject>,
    map_next_index: usize,
    #[unsafe_ignore_trace]
    map_iteration_kind: PropertyNameKind,
    lock: MapLock,
}

impl IntrinsicObject for MapIterator {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .iterator(),
            )
            .static_method(Self::next, js_string!("next"), 0)
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("Map Iterator"),
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().map()
    }
}

impl MapIterator {
    /// Abstract operation `CreateMapIterator( map, kind )`
    ///
    /// Creates a new iterator over the given map.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createmapiterator
    pub(crate) fn create_map_iterator(
        map: &JsValue,
        kind: PropertyNameKind,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        if let Some(map_obj) = map.as_object() {
            if let Some(map) = map_obj.borrow_mut().as_map_mut() {
                let lock = map.lock(map_obj.clone());
                let iter = Self {
                    iterated_map: Some(map_obj.clone()),
                    map_next_index: 0,
                    map_iteration_kind: kind,
                    lock,
                };
                let map_iterator = JsObject::from_proto_and_data_with_shared_shape(
                    context.root_shape(),
                    context.intrinsics().objects().iterator_prototypes().map(),
                    ObjectData::map_iterator(iter),
                );
                return Ok(map_iterator.into());
            }
        }
        Err(JsNativeError::typ()
            .with_message("`this` is not a Map")
            .into())
    }

    /// %MapIteratorPrototype%.next( )
    ///
    /// Advances the iterator and gets the next result in the map.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%mapiteratorprototype%.next
    pub(crate) fn next(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let mut map_iterator = this.as_object().map(JsObject::borrow_mut);
        let map_iterator = map_iterator
            .as_mut()
            .and_then(|obj| obj.as_map_iterator_mut())
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a MapIterator"))?;

        let item_kind = map_iterator.map_iteration_kind;

        if let Some(obj) = map_iterator.iterated_map.take() {
            let e = {
                let map = obj.borrow();
                let entries = map.as_map().expect("iterator should only iterate maps");
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
}
