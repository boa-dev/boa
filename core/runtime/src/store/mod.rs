//! Module containing the types related to the [`JsValueStore`].
use boa_engine::bigint::RawBigInt;
use boa_engine::builtins::error::ErrorKind;
use boa_engine::builtins::typed_array::TypedArrayKind;
use boa_engine::value::TryIntoJs;
use boa_engine::{Context, JsError, JsObject, JsResult, JsString, JsValue, js_error};
use std::collections::HashSet;
use std::sync::Arc;

mod from;
mod to;

/// Convenience method to avoid copy-pasting the same message.
#[inline]
fn unsupported_type() -> JsError {
    js_error!(Error: "DataCloneError: unsupported type for structured data")
}

/// A type to help store [`JsString`]. Because [`JsString`] relies on [`std::rc::Rc`],
/// it cannot be `Send`, which is a necessary contract for the Store. The [`StringStore`]
/// can be transformed from and into `JsString`, but owns its data. It is _not_ copy-on-
/// write.
#[derive(Debug, Eq, PartialEq, Hash)]
struct StringStore(Vec<u16>);

impl StringStore {
    fn to_js_string(&self) -> JsString {
        JsString::from(self.0.as_slice())
    }
}

impl From<JsString> for StringStore {
    fn from(value: JsString) -> Self {
        Self(value.to_vec())
    }
}

impl From<StringStore> for JsString {
    fn from(value: StringStore) -> Self {
        JsString::from(value.0.as_slice())
    }
}

/// Inner value for [`JsValueStore`].
#[derive(Debug, PartialEq)]
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
    String(StringStore),

    /// [`boa_engine::JsBigInt`]s are context-free but not `Send`. The Raw version
    /// of it is, though.
    BigInt(RawBigInt),

    /// A dictionary of strings to values which should be reconstructed into
    /// a `JsObject`. Note: the prototype and constructor are not maintained,
    /// and during reconstruction the default `Object` prototype will be used.
    Object(Vec<(StringStore, JsValueStore)>),

    /// A `Map()` object in JavaScript.
    Map(Vec<(JsValueStore, JsValueStore)>),

    /// A `Set()` object in JavaScript. The elements are already unique at
    /// construction.
    Set(Vec<JsValueStore>),

    /// An `Array` object in JavaScript.
    Array(Vec<Option<JsValueStore>>),

    /// A `Date` object in JavaScript. Although this can be marshaled, it uses
    /// the system's datetime library to be reconstructed and may diverge.
    #[expect(unused)]
    Date(std::time::Instant),

    /// Allowed error types (see the structured clone algorithm page).
    #[expect(unused)]
    Error {
        kind: ErrorKind,
        name: StringStore,
        message: StringStore,
        stack: StringStore,
        cause: StringStore,
    },

    /// Regular expression. We store the expression itself.
    #[expect(unused)]
    RegExp(StringStore),

    /// Array Buffer.
    ArrayBuffer(Vec<u8>),

    /// Dataview.
    #[expect(unused)]
    DataView {
        buffer: JsValueStore,
        byte_length: usize,
        byte_offset: usize,
    },

    /// Typed Array, including its kind and data.
    TypedArray {
        kind: TypedArrayKind,
        buffer: JsValueStore,
    },
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
/// pass in the context of the initial value. To transform it back to a
/// [`JsValue`], the application MUST pass the context that will contain
/// all prototypes for the new types (e.g. Object).
///
/// [sca]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm
#[derive(Debug, Clone, PartialEq)]
pub struct JsValueStore(Arc<ValueStoreInner>);

impl TryIntoJs for JsValueStore {
    fn try_into_js(&self, context: &mut Context) -> JsResult<JsValue> {
        let mut seen = to::ReverseSeenMap::default();
        to::try_value_into_js(self, &mut seen, context)
    }
}

impl JsValueStore {
    /// Replace the inner content with a new inner. This is necessary as the inner
    /// content holder must be allocated before its own inner content is created
    /// (to allow for recursive data). Therefore, the pattern is to create the
    /// store with an empty inner, then create the sub-content, and replace the
    /// empty inner with the new inner.
    ///
    /// # SAFETY
    /// This should only be done if the inner content is [`ValueStoreInner::Empty`],
    /// and only by the creator of the current [`JsValueStore`]. We enforce the first
    /// rule at runtime (and will panic), and the second rule by requiring a mutable
    /// reference. This is still unsafe and relies on unsafe pointer access.
    unsafe fn replace(&mut self, other: ValueStoreInner) {
        let ptr = Arc::as_ptr(&self.0).cast_mut();

        assert!(!ptr.is_null());
        unsafe {
            assert_eq!(
                *ptr,
                ValueStoreInner::Empty,
                "ValueStoreInner must be empty."
            );

            *ptr = other;
        }
    }

    /// A still-being-constructed value.
    fn empty() -> Self {
        Self(Arc::new(ValueStoreInner::Empty))
    }

    fn new(inner: ValueStoreInner) -> Self {
        Self(Arc::new(inner))
    }

    /// Create a context-free [`JsValue`] equivalent from an existing `JsValue` and the
    /// [`Context`] that was used to create it. The `transfer` argument allows for
    /// transferring ownership of the inner data to the context-free value instead of
    /// cloning it. By default, if a value isn't in the transfer vector, it is cloned.
    ///
    /// # Errors
    /// Any errors related to transferring or cloning a value's inner data.
    pub fn try_from_js(
        value: &JsValue,
        context: &mut Context,
        transfer: Vec<JsObject>,
    ) -> JsResult<Self> {
        let mut seen = from::SeenMap::default();
        let transfer = transfer.into_iter().collect::<HashSet<_>>();
        let v = from::try_from_js_value(value, &transfer, &mut seen, context)?;
        Ok(v)
    }
}
