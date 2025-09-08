//! All methods for serializing a [`JsValue`] into a [`JsValueStore`].

use crate::store::{JsValueStore, StringStore, ValueStoreInner, unsupported_type};
use boa_engine::builtins::array_buffer::ArrayBuffer;
use boa_engine::builtins::error::Error;
use boa_engine::object::builtins::{
    JsArray, JsArrayBuffer, JsDataView, JsDate, JsMap, JsRegExp, JsSet, JsTypedArray,
};
use boa_engine::property::PropertyKey;
use boa_engine::{Context, JsError, JsObject, JsResult, JsString, JsValue, JsVariant, js_error};
use std::collections::{HashMap, HashSet};

/// A Map of seen objects when walking through the value. We use the address
/// of the inner object as it is unique per JavaScript value.
#[derive(Default)]
pub(super) struct SeenMap(HashMap<usize, JsValueStore>);

impl SeenMap {
    fn get(&self, object: &JsObject) -> Option<JsValueStore> {
        let addr = std::ptr::from_ref(object.as_ref()).addr();
        self.0.get(&addr).cloned()
    }

    fn insert(&mut self, original: &JsObject, object: JsValueStore) {
        let addr = std::ptr::from_ref(original.as_ref()).addr();
        self.0.insert(addr, object);
    }
}

/// The core logic of the [`JsValueStore::try_from_js`] function.
fn try_from_js_object(
    value: &JsObject,
    transfer: &HashSet<JsObject>,
    seen: &mut SeenMap,
    context: &mut Context,
) -> JsResult<JsValueStore> {
    // Have we seen this object? If so, return its clone.
    if let Some(o2) = seen.get(value) {
        return Ok(o2.clone());
    }

    // Is it a transferable object?
    let new_value = if transfer.contains(value) {
        try_from_js_object_transfer(value, context)?
    } else {
        try_from_js_object_clone(value, transfer, seen, context)?
    };

    Ok(new_value)
}

/// Transfer an object into a store instead of cloning it. See [mdn].
///
/// Only [transferable objects][to] can be transferred. Anything else will return an
/// error. Since any object t
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects
/// [to]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects#supported_objects
fn try_from_js_object_transfer(
    object: &JsObject,
    _context: &mut Context,
) -> JsResult<JsValueStore> {
    if let Some(mut buffer) = object.downcast_mut::<ArrayBuffer>() {
        let data = buffer.detach(&JsValue::undefined())?;
        let data = data.ok_or_else(unsupported_type)?;

        Ok(JsValueStore::new(ValueStoreInner::ArrayBuffer(data)))
    } else {
        Err(unsupported_type())
    }
}

fn try_from_array_clone(
    array: &JsArray,
    transfer: &HashSet<JsObject>,
    seen: &mut SeenMap,
    context: &mut Context,
) -> JsResult<JsValueStore> {
    // Create an empty clone, we will replace its inner values after we gather them.
    // To stop the recursion, we need to add the right value to the seen map prior,
    // though.
    let mut dolly = JsValueStore::empty();
    seen.insert(&JsObject::from(array.clone()), dolly.clone());

    let length = array.length(context)?;
    let length = usize::try_from(length).map_err(JsError::from_rust)?;
    let mut inner = Vec::with_capacity(length);
    for i in 0..length {
        let v = array
            .borrow()
            .properties()
            .get(&i.into())
            .and_then(|x| x.value().cloned());
        if let Some(v) = v {
            let v = try_from_js_value(&v, transfer, seen, context)?;
            inner.push(Some(v));
        } else {
            inner.push(None);
        }
    }

    // SAFETY: This is safe as this function is the sole owner of the store.
    unsafe {
        dolly.replace(ValueStoreInner::Array(inner));
    }
    Ok(dolly)
}

fn try_from_array_buffer_clone(
    original: &JsObject,
    buffer: &JsArrayBuffer,
    seen: &mut SeenMap,
) -> JsResult<JsValueStore> {
    let data = buffer.data().ok_or_else(unsupported_type)?;
    let data = data.to_vec();
    let new_value = JsValueStore::new(ValueStoreInner::ArrayBuffer(data));
    seen.insert(original, new_value.clone());

    Ok(new_value)
}

fn clone_typed_array(
    original: &JsObject,
    buffer: &JsTypedArray,
    transfer: &HashSet<JsObject>,
    seen: &mut SeenMap,
    context: &mut Context,
) -> JsResult<JsValueStore> {
    let kind = buffer.kind().ok_or_else(unsupported_type)?;
    let buffer = buffer.buffer(context)?;
    let buffer = try_from_js_value(&buffer, transfer, seen, context)?;
    let dolly = JsValueStore::new(ValueStoreInner::TypedArray { kind, buffer });
    seen.insert(original, dolly.clone());
    Ok(dolly)
}

fn clone_date(
    original: &JsObject,
    date: &JsDate,
    seen: &mut SeenMap,
    context: &mut Context,
) -> JsResult<JsValueStore> {
    let msec_since_epoch = date
        .get_time(context)?
        .as_number()
        .ok_or_else(unsupported_type)?;

    let stored = JsValueStore::new(ValueStoreInner::Date(msec_since_epoch));
    seen.insert(original, stored.clone());
    Ok(stored)
}

fn clone_regexp(
    original: &JsObject,
    regexp: &JsRegExp,
    seen: &mut SeenMap,
    context: &mut Context,
) -> JsResult<JsValueStore> {
    let source = regexp.source(context)?;
    let flags = regexp.flags(context)?;

    let stored = JsValueStore::new(ValueStoreInner::RegExp { source, flags });
    seen.insert(original, stored.clone());
    Ok(stored)
}

fn try_from_map(
    original: &JsObject,
    map: &JsMap,
    transfer: &HashSet<JsObject>,
    seen: &mut SeenMap,
    context: &mut Context,
) -> JsResult<JsValueStore> {
    let mut new_map = Vec::new();
    let mut store = JsValueStore::new(ValueStoreInner::Empty);
    seen.insert(original, store.clone());

    map.for_each_native(|k, v| {
        let key = try_from_js_value(&k, transfer, seen, context)?;
        let value = try_from_js_value(&v, transfer, seen, context)?;
        new_map.push((key, value));

        Ok(())
    })?;

    // SAFETY: This is safe as this function is the sole owner of the store.
    unsafe {
        store.replace(ValueStoreInner::Map(new_map));
    }

    Ok(store)
}

fn try_from_set(
    original: &JsObject,
    set: &JsSet,
    transfer: &HashSet<JsObject>,
    seen: &mut SeenMap,
    context: &mut Context,
) -> JsResult<JsValueStore> {
    let mut new_set = Vec::new();
    let mut store = JsValueStore::new(ValueStoreInner::Empty);
    seen.insert(original, store.clone());

    set.for_each_native(|v| {
        let value = try_from_js_value(&v, transfer, seen, context)?;
        new_set.push(value);

        Ok(())
    })?;

    // SAFETY: This is safe as this function is the sole owner of the store.
    unsafe {
        store.replace(ValueStoreInner::Set(new_set));
    }

    Ok(store)
}

fn try_from_js_object_clone(
    object: &JsObject,
    transfer: &HashSet<JsObject>,
    seen: &mut SeenMap,
    context: &mut Context,
) -> JsResult<JsValueStore> {
    // If this is a special type of object, apply some special rules to it.
    // Described in
    // https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm#supported_types

    if let Ok(array) = JsArray::from_object(object.clone()) {
        return try_from_array_clone(&array, transfer, seen, context);
    } else if let Ok(map) = JsMap::from_object(object.clone()) {
        return try_from_map(object, &map, transfer, seen, context);
    } else if let Ok(set) = JsSet::from_object(object.clone()) {
        return try_from_set(object, &set, transfer, seen, context);
    } else if let Ok(ref buffer) = JsArrayBuffer::from_object(object.clone()) {
        return try_from_array_buffer_clone(object, buffer, seen);
    } else if let Ok(ref typed_array) = JsTypedArray::from_object(object.clone()) {
        return clone_typed_array(object, typed_array, transfer, seen, context);
    } else if let Ok(ref date) = JsDate::from_object(object.clone()) {
        return clone_date(object, date, seen, context);
    } else if let Ok(_error) = object.clone().downcast::<Error>() {
        return Err(js_error!(TypeError: "Errors are not supported yet."));
    } else if let Ok(ref regexp) = JsRegExp::from_object(object.clone()) {
        return clone_regexp(object, regexp, seen, context);
    } else if let Ok(_dataview) = JsDataView::from_object(object.clone()) {
        return Err(js_error!(TypeError: "Data views are not supported yet."));
    } else if object.is_callable() {
        // Functions are invalid.
        return Err(unsupported_type());
    }

    // Create a new object and add own properties to it. This does not preserve
    // the prototype (nor do we want to).
    let mut dolly = JsValueStore::empty();
    seen.insert(object, dolly.clone());

    let keys = object.own_property_keys(context)?;
    let mut fields: Vec<(StringStore, JsValueStore)> = Vec::with_capacity(keys.len());
    for k in keys {
        let value = object.get(k.clone(), context)?;
        let key = match k {
            PropertyKey::String(s) => s.into(),
            PropertyKey::Symbol(_) => return Err(unsupported_type()),
            PropertyKey::Index(i) => JsString::from(format!("{}", i.get())).into(),
        };

        let v = try_from_js_value(&value, transfer, seen, context)?;
        fields.push((key, v));
    }

    // SAFETY: This is safe as this function is the sole owner of the store.
    unsafe {
        dolly.replace(ValueStoreInner::Object(fields));
    }
    Ok(dolly)
}

pub(super) fn try_from_js_value(
    value: &JsValue,
    transfer: &HashSet<JsObject>,
    seen: &mut SeenMap,
    context: &mut Context,
) -> JsResult<JsValueStore> {
    match value.variant() {
        JsVariant::Null => Ok(JsValueStore::new(ValueStoreInner::Null)),
        JsVariant::Undefined => Ok(JsValueStore::new(ValueStoreInner::Undefined)),
        JsVariant::Boolean(b) => Ok(JsValueStore::new(ValueStoreInner::Boolean(b))),
        JsVariant::String(s) => Ok(JsValueStore::new(ValueStoreInner::String(s.into()))),
        JsVariant::Float64(f) => Ok(JsValueStore::new(ValueStoreInner::Float(f))),
        JsVariant::Integer32(i) => Ok(JsValueStore::new(ValueStoreInner::Float(f64::from(i)))),
        JsVariant::BigInt(b) => Ok(JsValueStore::new(ValueStoreInner::BigInt(
            b.as_inner().clone(),
        ))),
        JsVariant::Object(ref o) => try_from_js_object(o, transfer, seen, context),

        // Symbols cannot be transferred/cloned.
        JsVariant::Symbol(_) => Err(unsupported_type()),
    }
}
