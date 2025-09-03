//! All methods for deserializing a [`JsValueStore`] into a [`JsValue`].
use crate::store::{JsValueStore, StringStore, ValueStoreInner, unsupported_type};
use boa_engine::builtins::typed_array::TypedArrayKind;
use boa_engine::object::builtins::{
    JsArray, JsArrayBuffer, JsMap, JsSet, js_typed_array_from_kind,
};
use boa_engine::{Context, JsBigInt, JsObject, JsResult, JsValue};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct ReverseSeenMap(HashMap<usize, JsObject>);

impl ReverseSeenMap {
    fn get(&self, object: &JsValueStore) -> Option<JsObject> {
        let addr = std::ptr::from_ref(object.0.as_ref()).addr();
        self.0.get(&addr).cloned()
    }

    fn insert(&mut self, original: &JsValueStore, object: JsObject) {
        let addr = std::ptr::from_ref(original.0.as_ref()).addr();
        self.0.insert(addr, object);
    }
}

fn try_fields_into_js_object(
    store: &JsValueStore,
    fields: &Vec<(StringStore, JsValueStore)>,
    seen: &mut ReverseSeenMap,
    context: &mut Context,
) -> JsResult<JsValue> {
    let dolly = JsObject::with_object_proto(context.intrinsics());
    seen.insert(store, dolly.clone());

    for (k, v) in fields {
        let k = k.to_js_string();
        let value = try_value_into_js(v, seen, context)?;
        dolly.set(k, value, true, context)?;
    }
    Ok(JsValue::from(dolly))
}

fn try_items_into_js_array(
    store: &JsValueStore,
    items: &[Option<JsValueStore>],
    seen: &mut ReverseSeenMap,
    context: &mut Context,
) -> JsResult<JsValue> {
    let dolly = JsArray::new(context);
    seen.insert(store, dolly.clone().into());

    for (k, v) in items
        .iter()
        .enumerate()
        .filter_map(|(k, v)| v.as_ref().map(|v| (k, v)))
    {
        let value = try_value_into_js(v, seen, context)?;
        dolly.set(k, value, true, context)?;
    }
    Ok(JsValue::from(dolly))
}

fn try_into_js_array_buffer(
    store: &JsValueStore,
    data: &[u8],
    seen: &mut ReverseSeenMap,
    context: &mut Context,
) -> JsResult<JsValue> {
    let buffer = JsArrayBuffer::from_byte_block(data.to_vec(), context)?;
    let obj = JsObject::from(buffer);
    seen.insert(store, obj.clone());
    Ok(JsValue::from(obj))
}

fn try_into_js_typed_array(
    store: &JsValueStore,
    kind: TypedArrayKind,
    buffer: &JsValueStore,
    seen: &mut ReverseSeenMap,
    context: &mut Context,
) -> JsResult<JsValue> {
    let buffer = try_value_into_js(buffer, seen, context)?;
    let Some(buffer) = buffer.as_object() else {
        return Err(unsupported_type());
    };
    let buffer = JsArrayBuffer::from_object(buffer)?;
    let array = js_typed_array_from_kind(kind, buffer, context)?;
    if let Some(o) = array.as_object() {
        seen.insert(store, o);
    }
    Ok(array)
}

fn try_into_js_map(
    store: &JsValueStore,
    key_values: &[(JsValueStore, JsValueStore)],
    seen: &mut ReverseSeenMap,
    context: &mut Context,
) -> JsResult<JsValue> {
    let map = JsMap::new(context);
    seen.insert(store, map.clone().into());
    for (k, v) in key_values {
        let k = try_value_into_js(k, seen, context)?;
        let v = try_value_into_js(v, seen, context)?;
        map.set(k, v, context)?;
    }

    Ok(JsValue::from(map))
}

fn try_into_js_set(
    store: &JsValueStore,
    values: &[JsValueStore],
    seen: &mut ReverseSeenMap,
    context: &mut Context,
) -> JsResult<JsValue> {
    let set = JsSet::new(context);
    seen.insert(store, set.clone().into());
    for v in values {
        let v = try_value_into_js(v, seen, context)?;
        set.add(v, context)?;
    }

    Ok(JsValue::from(set))
}

pub(super) fn try_value_into_js(
    store: &JsValueStore,
    seen: &mut ReverseSeenMap,
    context: &mut Context,
) -> JsResult<JsValue> {
    if let Some(v) = seen.get(store) {
        return Ok(JsValue::from(v));
    }

    // Match the value
    match &*store.0 {
        ValueStoreInner::Empty => {
            unreachable!("ValueStoreInner::Empty should not exist after storage.");
        }
        ValueStoreInner::Null => Ok(JsValue::null()),
        ValueStoreInner::Undefined => Ok(JsValue::undefined()),
        ValueStoreInner::Boolean(b) => Ok(JsValue::from(*b)),
        ValueStoreInner::Float(f) => Ok(JsValue::from(*f)),
        ValueStoreInner::String(s) => Ok(JsValue::from(s.to_js_string())),
        ValueStoreInner::BigInt(b) => Ok(JsValue::from(JsBigInt::new(b.clone()))),
        ValueStoreInner::Object(fields) => try_fields_into_js_object(store, fields, seen, context),
        ValueStoreInner::Map(key_values) => try_into_js_map(store, key_values, seen, context),
        ValueStoreInner::Set(values) => try_into_js_set(store, values, seen, context),
        ValueStoreInner::Array(items) => try_items_into_js_array(store, items, seen, context),
        ValueStoreInner::Date(_) => unimplemented!(),
        ValueStoreInner::Error { .. } => unimplemented!(),
        ValueStoreInner::RegExp(_) => unimplemented!(),
        ValueStoreInner::ArrayBuffer(data) => try_into_js_array_buffer(store, data, seen, context),
        ValueStoreInner::DataView { .. } => {
            unimplemented!()
        }
        ValueStoreInner::TypedArray { kind, buffer } => {
            try_into_js_typed_array(store, *kind, buffer, seen, context)
        }
    }
}
