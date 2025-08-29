//! Module containing all types and functions to implement `structuredClone`.
//!
//! See <https://developer.mozilla.org/en-US/docs/Web/API/Window/structuredClone>.
#![allow(clippy::needless_pass_by_value)]

use boa_engine::bigint::RawBigInt;
use boa_engine::builtins::array_buffer::ArrayBuffer;
use boa_engine::builtins::error::ErrorKind;
use boa_engine::builtins::typed_array::TypedArrayKind;
use boa_engine::object::builtins::{JsArray, JsArrayBuffer, JsTypedArray};
use boa_engine::property::PropertyKey;
use boa_engine::realm::Realm;
use boa_engine::value::{TryFromJs, TryIntoJs};
use boa_engine::{
    Context, JsBigInt, JsError, JsObject, JsResult, JsString, JsValue, JsVariant, js_error,
};
use boa_interop::boa_macros::boa_module;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;

/// Convenience method to avoid copy-pasting the same message.
#[inline]
fn unsupported_type() -> JsError {
    js_error!(Error: "DataCloneError: unsupported type for structured data")
}

#[derive(Default)]
struct SeenMap(HashMap<usize, JsValueStore>);

impl SeenMap {
    fn get(&self, object: &JsObject) -> Option<JsValueStore> {
        let addr = std::ptr::from_ref(object.as_ref()).addr();
        self.0.get(&addr).map(|s| s.clone())
    }

    fn insert(&mut self, original: &JsObject, object: JsValueStore) {
        let addr = std::ptr::from_ref(original.as_ref()).addr();
        self.0.insert(addr, object);
    }
}

#[derive(Default)]
struct ReverseSeenMap(HashMap<usize, JsObject>);

impl ReverseSeenMap {
    fn get(&self, object: &JsValueStore) -> Option<JsObject> {
        let addr = std::ptr::from_ref(object.0.as_ref()).addr();
        self.0.get(&addr).map(|s| s.clone())
    }

    fn insert(&mut self, original: &JsValueStore, object: JsObject) {
        let addr = std::ptr::from_ref(original.0.as_ref()).addr();
        self.0.insert(addr, object);
    }
}

/// Inner value for [`JsValueStore`].
#[derive(Debug, Clone)]
enum ValueStoreInner {
    /// An Empty value that will be filled later. This is only used during
    /// construction, and if encountered at other points will result
    /// in an error.
    Empty,

    /// Primitive values - `null`.
    Null,

    /// Primitive values - `undefined`.
    Undefined,

    /// Primitive values - `Boolean`.
    Boolean(bool),

    /// Primitive values - `float64`. No need to store integers separately,
    /// they'll be checked when recreating the `JsValue`.
    Float(f64),

    /// [`JsString`]s are context-free, but not `Send`. Since we want to be
    /// `Send`, we'll have to make a copy of the data.
    String(Vec<u16>),

    /// [`JsBigInt`]s are context-free but not `Send`. The Raw version of it
    /// is though.
    BigInt(RawBigInt),

    /// A dictionary of strings to values which should be reconstructed into
    /// a `JsObject`. Note: the prototype and constructor are not maintained,
    /// and during reconstruction the default `Object` prototype will be used.
    Object(HashMap<JsString, JsValueStore>),

    /// A `Map()` object in JavaScript.
    Map(HashMap<JsValueStore, JsValueStore>),

    /// A `Set()` object in JavaScript. The elements are already unique at
    /// construction.
    Set(Vec<JsValueStore>),

    /// An `Array` object in JavaScript.
    Array(Vec<Option<JsValueStore>>),

    /// A `Date` object in JavaScript. Although this can be marshalled, it uses
    /// the system's datetime library to be reconstructed and may diverge.
    Date(std::time::Instant),

    /// Allowed error types (see the structured clone algorithm page).
    Error {
        kind: ErrorKind,
        name: JsString,
        message: JsString,
        stack: JsString,
        cause: JsString,
    },

    /// Regular expression. We store the expression itself.
    RegExp(JsString),

    /// Array Buffer.
    ArrayBuffer(Vec<u8>),

    /// Dataview.
    DataView {
        buffer: JsValueStore,
        byte_length: usize,
        byte_offset: usize,
    },

    /// Typed Array, including its kind and data.
    TypedArray(TypedArrayKind, Vec<u8>),
}

impl ValueStoreInner {
    fn replace(&mut self, other: ValueStoreInner) {
        if let ValueStoreInner::Empty = self {
            *self = other;
        } else {
            unreachable!("Only empty inner values should be replaced");
        }
    }
}

/// A [`JsValue`]-like structure that can rebuild its value given any [`Context`].
/// It essentially stores the value itself and its original type. During
/// reconstruction, the constructors of the new [`Context`] will be used.
///
/// This follows the rules of the [structured clone algorithm][sca], but does not
/// require a [`Context`] to copy/move, and can be [`Send`] between threads.
///
/// It is not serializable as it allows recursive values.
///
/// To transform a [`JsValue`] into a [`JsValueStore`], the application MUST
/// pass in the context of the initial value.
///
/// [sca]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm
#[derive(Debug, Clone)]
pub struct JsValueStore(Arc<RefCell<ValueStoreInner>>);

impl TryIntoJs for JsValueStore {
    fn try_into_js(&self, context: &mut Context) -> JsResult<JsValue> {
        let mut seen = ReverseSeenMap::default();
        Self::try_value_into_js(self, &mut seen, context)
    }
}

// All methods for serializing a JsValue into a JsValueStore.
impl JsValueStore {
    /// The core logic of the [`JsValueStore::try_from_js`] function.
    fn try_from_js_object(
        value: JsObject,
        transfer: &HashSet<JsObject>,
        seen: &mut SeenMap,
        context: &mut Context,
    ) -> JsResult<JsValueStore> {
        // Have we seen this object? If so, return its clone.
        if let Some(o2) = seen.get(&value) {
            return Ok(o2.clone());
        }

        // Is it a transferable object?
        let new_value = if transfer.contains(&value) {
            Self::try_from_js_object_transfer(value.clone(), context)?
        } else {
            Self::try_from_js_object_clone(value.clone(), transfer, seen, context)?
        };

        seen.insert(&value, new_value.clone());
        Ok(new_value)
    }

    /// Transfer an object into a store, instead of cloning it. See [mdn].
    ///
    /// Only [transferable objects][to] can be transferred. Anything else will return an
    /// error. Since any object t
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects
    /// [to]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects#supported_objects
    fn try_from_js_object_transfer(
        object: JsObject,
        context: &mut Context,
    ) -> JsResult<JsValueStore> {
        if let Some(mut buffer) = object.clone().downcast_mut::<ArrayBuffer>() {
            let data = buffer.detach(&JsValue::undefined())?;
            let data = data.ok_or_else(unsupported_type)?;

            Ok(JsValueStore::new(ValueStoreInner::ArrayBuffer(data)))
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
            Ok(JsValueStore::new(ValueStoreInner::TypedArray(kind, data)))
        } else {
            Err(unsupported_type())
        }
    }

    fn try_from_array_clone(
        array: JsArray,
        transfer: &HashSet<JsObject>,
        seen: &mut SeenMap,
        context: &mut Context,
    ) -> JsResult<JsValueStore> {
        // Create an empty clone, we will replace its inner values after we gather them.
        // To stop the recursion, we need to add the right value to the seen map prior,
        // though.
        let dolly = JsValueStore::empty();
        seen.insert(&JsObject::from(array.clone()), dolly.clone());

        let length = array.length(context)?;
        let mut inner = Vec::with_capacity(length as usize);
        for i in 0..length {
            let v = array
                .borrow()
                .properties()
                .get(&i.into())
                .and_then(|x| x.value().cloned());
            if let Some(v) = v {
                let v = Self::try_from_js_inner(&v, context, transfer, seen)?;
                inner.push(Some(v));
            } else {
                inner.push(None);
            }
        }

        dolly.0.borrow_mut().replace(ValueStoreInner::Array(inner));
        Ok(dolly)
    }

    fn try_from_array_buffer_clone(
        original: &JsObject,
        buffer: JsArrayBuffer,
        seen: &mut SeenMap,
    ) -> JsResult<JsValueStore> {
        let data = buffer.data().ok_or_else(unsupported_type)?;
        let data = data.to_vec();
        let new_value = Self::new(ValueStoreInner::ArrayBuffer(data));
        seen.insert(original, new_value.clone());

        Ok(new_value)
    }

    fn clone_typed_array(
        original: &JsObject,
        buffer: JsTypedArray,
        seen: &mut SeenMap,
        context: &mut Context,
    ) -> JsResult<JsValueStore> {
        let kind = buffer.kind().ok_or_else(unsupported_type)?;
        let buffer = buffer
            .buffer(context)?
            .as_object()
            .ok_or_else(unsupported_type)?;
        let buffer = buffer
            .downcast_mut::<ArrayBuffer>()
            .ok_or_else(unsupported_type)?;
        let data = buffer.data().ok_or_else(unsupported_type)?;
        let dolly = Self::new(ValueStoreInner::TypedArray(kind, data.to_vec()));
        seen.insert(original, dolly.clone());
        Ok(dolly)
    }

    fn try_from_js_object_clone(
        object: JsObject,
        transfer: &HashSet<JsObject>,
        seen: &mut SeenMap,
        context: &mut Context,
    ) -> JsResult<JsValueStore> {
        // If this is a special type of object, apply some special rules to it.
        // Described in
        // https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm#supported_types

        if let Ok(array) = JsArray::from_object(object.clone()) {
            return Self::try_from_array_clone(array, transfer, seen, context);
        }
        if let Ok(buffer) = JsArrayBuffer::from_object(object.clone()) {
            return Self::try_from_array_buffer_clone(&object, buffer, seen);
        }
        if let Ok(typed_array) = JsTypedArray::from_object(object.clone()) {
            return Self::clone_typed_array(&object, typed_array, seen, context);
        }

        // Functions are invalid.
        if object.is_callable() {
            return Err(unsupported_type());
        }

        // Create a new object and add own properties to it. This does not preserve
        // the prototype (nor do we want to).
        let dolly = Self::empty();
        seen.insert(&object, dolly.clone());

        let mut fields: HashMap<JsString, JsValueStore> = HashMap::new();
        let keys = object.own_property_keys(context)?;
        for k in keys {
            let value = object.get(k.clone(), context)?;
            let key = match k {
                PropertyKey::String(s) => s,
                PropertyKey::Symbol(_) => return Err(unsupported_type()),
                PropertyKey::Index(i) => JsString::from(format!("{}", i.get())),
            };

            //Self::try_from_js_inner(k, context, transfer, seen)?;
            let v = Self::try_from_js_inner(&value, context, transfer, seen)?;
            fields.insert(key, v);
        }

        dolly.0.replace(ValueStoreInner::Object(fields));
        Ok(dolly)
    }

    fn try_from_js_inner(
        value: &JsValue,
        context: &mut Context,
        transfer: &HashSet<JsObject>,
        seen: &mut SeenMap,
    ) -> JsResult<JsValueStore> {
        match value.variant() {
            JsVariant::Null => Ok(Self::new(ValueStoreInner::Null)),
            JsVariant::Undefined => Ok(Self::new(ValueStoreInner::Undefined)),
            JsVariant::Boolean(b) => Ok(Self::new(ValueStoreInner::Boolean(b))),
            JsVariant::String(s) => Ok(Self::new(ValueStoreInner::String(s.to_vec()))),
            JsVariant::Float64(f) => Ok(Self::new(ValueStoreInner::Float(f))),
            JsVariant::Integer32(i) => Ok(Self::new(ValueStoreInner::Float(i as f64))),
            JsVariant::BigInt(b) => Ok(Self::new(ValueStoreInner::BigInt(b.as_inner().clone()))),
            JsVariant::Object(o) => Self::try_from_js_object(o, transfer, seen, context),

            // Symbols cannot be transferred/cloned.
            JsVariant::Symbol(_) => Err(unsupported_type()),
        }
    }

    /// A still-being-constructed value.
    fn empty() -> Self {
        Self(Arc::new(RefCell::new(ValueStoreInner::Empty)))
    }

    fn new(inner: ValueStoreInner) -> Self {
        Self(Arc::new(RefCell::new(inner)))
    }

    /// Create a context-free [`JsValue`] equivalent from an existing `JsValue` and the
    /// [`Context`] that was used to create it. The `transfer` argument allows for
    /// transferring ownership of the inner data to the context-free value, instead of
    /// cloning it. By default, if a value isn't in the transfer vector, it is cloned.
    pub fn try_from_js(
        value: &JsValue,
        context: &mut Context,
        transfer: Vec<JsObject>,
    ) -> JsResult<Self> {
        let mut seen = SeenMap::default();
        let transfer = transfer.into_iter().collect::<HashSet<_>>();
        let v = Self::try_from_js_inner(value, context, &transfer, &mut seen)?;
        Ok(v)
    }
}

// All methods for deserializing a JsValueStore into a JsValue.
impl JsValueStore {
    fn try_fields_into_js_object(
        &self,
        fields: &HashMap<JsString, JsValueStore>,
        seen: &mut ReverseSeenMap,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let dolly = JsObject::with_object_proto(context.intrinsics());
        seen.insert(self, dolly.clone());

        for (k, v) in fields {
            let value = v.try_value_into_js(seen, context)?;
            dolly.set(k.clone(), value, true, context)?;
        }
        Ok(JsValue::from(dolly))
    }

    fn try_items_into_js_array(
        &self,
        items: &Vec<Option<JsValueStore>>,
        seen: &mut ReverseSeenMap,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let dolly = JsArray::new(context);
        seen.insert(self, dolly.clone().into());

        for (k, v) in items
            .iter()
            .enumerate()
            .filter_map(|(k, v)| if let Some(v) = v { Some((k, v)) } else { None })
        {
            let value = v.try_value_into_js(seen, context)?;
            dolly.set(k, value, true, context)?;
        }
        Ok(JsValue::from(dolly))
    }

    fn try_value_into_js(
        &self,
        seen: &mut ReverseSeenMap,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let Some(v) = seen.get(self) {
            return Ok(JsValue::from(v));
        }

        // Match the value
        match self.0.borrow().deref() {
            ValueStoreInner::Empty => {
                unreachable!("ValueStoreInner::Empty should not exist after storage.");
            }
            ValueStoreInner::Null => Ok(JsValue::null()),
            ValueStoreInner::Undefined => Ok(JsValue::undefined()),
            ValueStoreInner::Boolean(b) => Ok(JsValue::from(*b)),
            ValueStoreInner::Float(f) => Ok(JsValue::from(*f)),
            ValueStoreInner::String(s) => Ok(JsValue::from(JsString::from(s.as_slice()))),
            ValueStoreInner::BigInt(b) => Ok(JsValue::from(JsBigInt::new(b.clone()))),
            ValueStoreInner::Object(fields) => {
                Self::try_fields_into_js_object(self, fields, seen, context)
            }
            ValueStoreInner::Map(_) => {
                todo!()
            }
            ValueStoreInner::Set(_) => todo!(),
            ValueStoreInner::Array(items) => {
                Self::try_items_into_js_array(self, items, seen, context)
            }
            ValueStoreInner::Date(_) => todo!(),
            ValueStoreInner::Error { .. } => todo!(),
            ValueStoreInner::RegExp(_) => todo!(),
            ValueStoreInner::ArrayBuffer(_) => {
                todo!()
            }
            ValueStoreInner::DataView { .. } => {
                todo!()
            }
            ValueStoreInner::TypedArray(_, _) => {
                todo!()
            }
        }
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
    use super::{JsValueStore, StructuredCloneOptions};
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
        let v = JsValueStore::try_from_js(
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
