//! Module containing all types and functions to implement `structuredClone`.
//!
//! See <https://developer.mozilla.org/en-US/docs/Web/API/Window/structuredClone>.
#![allow(clippy::needless_pass_by_value)]

use boa_engine::builtins::array_buffer::ArrayBuffer;
use boa_engine::builtins::error::ErrorKind;
use boa_engine::builtins::regexp::RegExp;
use boa_engine::builtins::typed_array::TypedArrayKind;
use boa_engine::object::builtins::{JsArray, JsArrayBuffer, JsTypedArray};
use boa_engine::realm::Realm;
use boa_engine::value::{TryFromJs, TryIntoJs};
use boa_engine::{Context, JsError, JsObject, JsResult, JsString, JsValue, js_error};
use boa_interop::boa_macros::boa_module;
use std::collections::HashMap;
use std::sync::Arc;

#[inline]
fn unsupported_type() -> JsError {
    js_error!(Error: "DataCloneError: unsupported type for structured data")
}

/// Inner value for [`ContextFreeJsValue`].
#[derive(Clone)]
enum ContextFreeValueInner {
    /// A primitive value that does not require a context to recreate. Includes
    /// booleans, null, floats, etc.
    Primitive(Arc<JsValue>),

    /// A reference to another inner within the same value tree. This is to
    /// allow recursive data structures.
    Ref(Arc<ContextFreeValueInner>),

    /// A dictionary of strings to values which should be reconstructed into
    /// a `JsObject`. Note: the prototype and constructor are not maintained,
    /// and during reconstruction the default `Object` prototype will be used.
    Object(HashMap<String, Arc<ContextFreeValueInner>>),

    /// A `Map()` object in JavaScript.
    Map(HashMap<String, Arc<ContextFreeValueInner>>),

    /// A `Set()` object in JavaScript. The elements are already unique at
    /// construction.
    Set(Arc<Vec<ContextFreeValueInner>>),

    /// An `Array` object in JavaScript.
    Array(Arc<Vec<ContextFreeValueInner>>),

    /// A `Date` object in JavaScript. Although this can be marshalled, it uses
    /// the system's datetime library to be reconstructed and may diverge.
    Date(Arc<std::time::Instant>),

    /// Allowed error types (see the structured clone algorithm page).
    Error {
        kind: ErrorKind,
        name: JsString,
        message: JsString,
        stack: JsString,
        cause: JsString,
    },

    /// Regular expression.
    RegExp(Arc<JsString>),

    /// Array Buffer and co.
    ArrayBuffer(Arc<Vec<u8>>),
    DataView {
        buffer: Arc<ContextFreeValueInner>,
        byte_length: usize,
        byte_offset: usize,
    },
    TypedArray(TypedArrayKind, Arc<Vec<u8>>),
}

/// A [`JsValue`]-like structure that can rebuild its value given any [`Context`].
/// This follows the rules of the [structured clone algorithm][sca], but does not
/// require a [`Context`] to copy/move, and is [`Send`].
///
/// To deserialize a [`JsValue`] into a [`ContextFreeJsValue`], the application MUST
/// pass in the context of the initial value.
///
/// [sca]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm
#[derive(Clone)]
pub struct ContextFreeJsValue {
    inner: ContextFreeValueInner,
}

impl TryIntoJs for ContextFreeJsValue {
    fn try_into_js(&self, context: &mut Context) -> JsResult<JsValue> {
        todo!()
    }
}

impl ContextFreeJsValue {
    /// Create a context-free [`JsValue`] equivalent from an existing `JsValue` and the
    /// [`Context`] that was used to create it. The `transfer` argument allows for
    /// transferring ownership of the inner data to the context-free value, instead of
    /// cloning it. By default, if a value isn't in the transfer vector, it is cloned.
    pub fn try_from_js(
        value: &JsValue,
        context: &mut Context,
        transfer: Vec<JsObject>,
    ) -> JsResult<Self> {
    }
}

/// Transfer an object instead of cloning it. See [mdn].
///
/// Only [transferable objects][to] can be transferred. Anything else will return an
/// error.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects
/// [to]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects#supported_objects
fn transfer_object(object: JsObject, context: &mut Context) -> JsResult<JsValue> {
    if let Some(mut buffer) = object.clone().downcast_mut::<ArrayBuffer>() {
        let data = buffer.detach(&JsValue::undefined())?;
        let data = data.ok_or_else(unsupported_type)?;

        JsArrayBuffer::from_byte_block(data, context).map(Into::into)
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
        boa_engine::object::builtins::js_typed_array_from_kind(kind, buffer, context)
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
    let data = buffer.data().ok_or_else(unsupported_type)?;
    JsArrayBuffer::from_byte_block(data.to_vec(), context).map(Into::into)
}

fn clone_typed_array(buffer: JsTypedArray, context: &mut Context) -> JsResult<JsValue> {
    let kind = buffer.kind().ok_or_else(unsupported_type)?;
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
    boa_engine::object::builtins::js_typed_array_from_kind(kind, dolly, context)
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
        return Ok(o2.clone());
    }

    // Is it a transferable object?
    if Some(true)
        == options
            .and_then(|o| o.transfer.as_ref())
            .map(|t| t.contains(&o))
    {
        let v = transfer_object(o, context)?;
        seen.insert(value.clone(), v.clone());
        Ok(v)
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
    use super::{ContextFreeJsValue, StructuredCloneOptions};
    use boa_engine::value::TryIntoJs;
    use boa_engine::{Context, JsResult, JsValue};

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
        let v = ContextFreeJsValue::try_from_js(
            &value,
            context,
            options.and_then(|o| o.transfer).unwrap_or_default(),
        )?;
        v.try_into_js(context)
    }
}

/// Register the `structuredClone` function in the global context.
///
/// # Errors
/// Return an error if the function is already registered.
pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
    js_module::boa_register(realm, context)
}
