//! Module containing all types and functions to implement `structuredClone`.
//!
//! See <https://developer.mozilla.org/en-US/docs/Web/API/Window/structuredClone>.
#![allow(clippy::needless_pass_by_value)]

use boa_engine::builtins::array_buffer::ArrayBuffer;
use boa_engine::object::builtins::{JsArray, JsArrayBuffer, JsTypedArray, JsUint8Array};
use boa_engine::realm::Realm;
use boa_engine::value::TryFromJs;
use boa_engine::{js_error, Context, JsError, JsObject, JsResult, JsValue};
use boa_interop::boa_macros::boa_module;
use std::collections::HashMap;

#[inline]
fn unsupported_type() -> JsError {
    js_error!(Error: "DataCloneError: unsupported type for structured data")
}

/// Transfer an object instead of cloning it. See [mdn].
///
/// Only [transferable objects][to] can be transferred. Anything else will return an
/// error.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects
/// [to]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects#supported_objects
fn transfer_object(
    value: &JsValue,
    object: JsObject,
    _options: Option<&StructuredCloneOptions>,
    seen: &mut HashMap<JsValue, JsValue>,
    context: &mut Context,
) -> JsResult<JsValue> {
    if let Some(mut buffer) = object.clone().downcast_mut::<ArrayBuffer>() {
        let data = buffer.detach(&JsValue::undefined())?;
        let data = data.ok_or_else(
            || js_error!(Error: "DataCloneError: unsupported type for structured data"),
        )?;

        let dolly = JsUint8Array::from_array_buffer(
            JsArrayBuffer::from_byte_block(data, context)?,
            context,
        )?;
        let v: JsValue = dolly.into();
        seen.insert(value.clone(), v.clone());
        Ok(v)
    } else if let Ok(typed_array) = JsTypedArray::from_object(object) {
        let kind = typed_array.kind().ok_or_else(unsupported_type)?;
        let data = typed_array.buffer(context)?;
        let buffer = data.as_object().ok_or_else(unsupported_type)?;
        let mut buffer = buffer
            .downcast_mut::<ArrayBuffer>()
            .ok_or_else(unsupported_type)?;
        let data = buffer.detach(&JsValue::undefined())?.ok_or_else(
            || js_error!(Error: "DataCloneError: unsupported type for structured data"),
        )?;

        let buffer = JsArrayBuffer::from_byte_block(data, context)?;
        let dolly = boa_engine::object::builtins::js_typed_array_from_kind(kind, buffer, context)?;
        let v: JsValue = dolly.into();
        seen.insert(value.clone(), v.clone());
        Ok(v)
    } else {
        Err(unsupported_type())
    }
}

fn clone_array(
    array: JsArray,
    options: Option<&StructuredCloneOptions>,
    seen: &mut HashMap<JsValue, JsValue>,
    context: &mut Context,
) -> JsResult<JsValue> {
    let dolly = JsArray::new(context);
    seen.insert(array.clone().into(), dolly.clone().into());

    let length = array.length(context)?;
    for i in 0..length {
        let v = array.get(i, context)?;
        let v = structured_clone_inner(&v, options, seen, context)?;
        dolly.push(v, context)?;
    }

    Ok(dolly.into())
}

fn clone_array_buffer(buffer: JsArrayBuffer, context: &mut Context) -> JsResult<JsValue> {
    let dolly = {
        let data = buffer.data().ok_or_else(unsupported_type)?;
        JsArrayBuffer::from_byte_block(data.to_vec(), context)?;
    };

    Ok(dolly.into())
}

fn clone_typed_array(buffer: JsTypedArray, context: &mut Context) -> JsResult<JsValue> {
    let dolly = {
        let buffer = buffer
            .buffer(context)?
            .as_object()
            .ok_or_else(unsupported_type)?;
        let buffer = buffer
            .downcast_mut::<ArrayBuffer>()
            .ok_or_else(unsupported_type)?;
        let data = buffer.data().ok_or_else(unsupported_type)?;
        JsArrayBuffer::from_byte_block(data.to_vec(), context)?
    };
    JsUint8Array::from_array_buffer(dolly, context).map(|a| a.into())
}

fn clone_object(
    value: &JsValue,
    object: JsObject,
    options: Option<&StructuredCloneOptions>,
    seen: &mut HashMap<JsValue, JsValue>,
    context: &mut Context,
) -> JsResult<JsValue> {
    // If this is a special type of object, apply some special rules to it.
    // Described in
    // https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm#supported_types

    if let Ok(array) = JsArray::from_object(object.clone()) {
        return clone_array(array, options, seen, context);
    }
    if let Ok(buffer) = JsArrayBuffer::from_object(object.clone()) {
        let v = clone_array_buffer(buffer, context)?;
        seen.insert(value.clone(), v.clone());
        return Ok(v);
    }
    if let Ok(typed_array) = JsTypedArray::from_object(object.clone()) {
        let v = clone_typed_array(typed_array, context)?;
        eprintln!("inserting {v:?}");
        seen.insert(value.clone(), v.clone());
        return Ok(v);
    }

    // Functions are invalid.
    if object.is_callable() {
        return Err(unsupported_type());
    }

    // Create a new object and add own properties to it. This does not preserve
    // the prototype (nor do we want to).
    let dolly = JsObject::with_object_proto(context.intrinsics());
    seen.insert(value.clone(), dolly.clone().into());

    let keys = object.own_property_keys(context)?;
    for k in keys {
        let value = object.get(k.clone(), context)?;
        eprintln!("{k} => {value:?}");
        let v = structured_clone_inner(&value, options, seen, context)?;
        dolly.set(k, v, true, context)?;
    }

    Ok(dolly.into())
}

/// The core logic of the `structuredClone` function.
fn structured_clone_inner(
    value: &JsValue,
    options: Option<&StructuredCloneOptions>,
    seen: &mut HashMap<JsValue, JsValue>,
    context: &mut Context,
) -> JsResult<JsValue> {
    // If the value is not an object or object-like, just clone it.
    let Some(o) = value.as_object() else {
        // Except symbols... Those are not cloneable.
        if value.is_symbol() {
            return Err(unsupported_type());
        }
        return Ok(value.clone());
    };

    // Have we seen this object? If so, return its clone.
    if let Some(o2) = seen.get(value) {
        eprintln!("seen? {:?}", seen.get(value));
        return Ok(o2.clone());
    }

    // Is it a transferable object?
    if Some(true)
        == options
            .and_then(|o| o.transfer.as_ref())
            .map(|t| t.contains(&o))
    {
        transfer_object(value, o, options, seen, context)
    } else {
        clone_object(value, o, options, seen, context)
    }
}

/// Options used by `structuredClone`. This is currently unused.
#[derive(Debug, Clone, TryFromJs)]
pub struct StructuredCloneOptions {
    transfer: Option<Vec<JsObject>>,
}

/// JavaScript module containing the `structuredClone` types and functions.
#[boa_module]
pub mod js_module {
    use super::StructuredCloneOptions;
    use boa_engine::{Context, JsResult, JsValue};
    use std::collections::HashMap;

    /// The [`structuredClone()`][mdn] method of the Window interface creates a
    /// deep clone of a given value using the [structured clone algorithm][sca].
    ///
    /// # Errors
    /// Will return an error if the context cannot create objects or copy bytes, or
    /// if any unhandled case by the structured clone algorithm.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Window/structuredClone
    /// [sca]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm
    pub fn structured_clone(
        value: JsValue,
        options: Option<StructuredCloneOptions>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Recursive method to clone.
        super::structured_clone_inner(&value, options.as_ref(), &mut HashMap::new(), context)
    }
}

/// Register the `structuredClone` function in the global context.
///
/// # Errors
/// Return an error if the function is already registered.
pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
    js_module::boa_register(realm, context)
}
