//! Module implementing JavaScript classes to handle text encoding and decoding.
//!
//! See <https://developer.mozilla.org/en-US/docs/Web/API/Encoding_API> for more information.

use boa_engine::object::builtins::{JsArrayBuffer, JsTypedArray, JsUint8Array};
use boa_engine::realm::Realm;
use boa_engine::value::TryFromJs;
use boa_engine::{
    Context, Finalize, JsData, JsObject, JsResult, JsString, JsValue, Trace, boa_class, boa_module,
    js_error, js_string,
};

#[cfg(test)]
mod tests;

mod encodings;

/// The [`TextDecoder`][mdn] class represents an encoder for a specific method, that is
/// a specific character encoding, like `utf-8`.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder
#[derive(Debug, Default, Clone, JsData, Trace, Finalize)]
pub enum TextDecoder {
    /// Decode bytes encoded as UTF-8 into strings.
    #[default]
    Utf8,
    /// Decode bytes encoded as UTF-16 (little endian) into strings.
    Utf16Le,
    /// Decode bytes encoded as UTF-16 (big endian) into strings.
    Utf16Be,
}

#[boa_class]
impl TextDecoder {
    /// The [`TextDecoder()`][mdn] constructor returns a new `TextDecoder` object.
    ///
    /// # Errors
    /// This will return an error if the encoding or options are invalid or unsupported.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder
    #[boa(constructor)]
    pub fn constructor(encoding: Option<JsString>, _options: Option<JsObject>) -> JsResult<Self> {
        let Some(encoding) = encoding else {
            return Ok(Self::default());
        };

        match encoding.to_std_string_lossy().as_str() {
            "utf-8" => Ok(Self::Utf8),
            // Default encoding is Little Endian.
            "utf-16" | "utf-16le" => Ok(Self::Utf16Le),
            "utf-16be" => Ok(Self::Utf16Be),
            e => Err(js_error!(RangeError: "The given encoding '{}' is not supported.", e)),
        }
    }

    /// The [`TextDecoder.encoding`][mdn] read-only property returns a string containing
    /// the name of the character encoding that this decoder will use.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/encoding
    #[boa(getter)]
    #[must_use]
    pub fn encoding(&self) -> JsString {
        match self {
            Self::Utf8 => js_string!("utf-8"),
            Self::Utf16Le => js_string!("utf-16le"),
            Self::Utf16Be => js_string!("utf-16be"),
        }
    }

    /// The [`TextDecoder.decode()`][mdn] method returns a string containing text decoded from the
    /// buffer passed as a parameter.
    ///
    /// # Errors
    /// Any error that arises during decoding the specific encoding.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/decode
    pub fn decode(&self, buffer: JsValue, context: &mut Context) -> JsResult<JsString> {
        // `buffer` can be an `ArrayBuffer`, a `TypedArray` or a `DataView`.
        let array_buffer = if let Ok(array_buffer) = JsArrayBuffer::try_from_js(&buffer, context) {
            array_buffer
        } else if let Ok(typed_array) = JsTypedArray::try_from_js(&buffer, context) {
            let Some(obj) = typed_array.buffer(context)?.as_object() else {
                return Err(js_error!(TypeError: "Invalid buffer backing TypedArray."));
            };

            JsArrayBuffer::from_object(obj)?
        } else {
            return Err(js_error!(
                TypeError: "Argument 1 must be an ArrayBuffer, TypedArray or DataView."
            ));
        };

        let Some(data) = array_buffer.data() else {
            return Err(js_error!(TypeError: "Detached ArrayBuffer"));
        };

        Ok(match self {
            Self::Utf8 => encodings::utf8::decode(&data),
            Self::Utf16Le => encodings::utf16le::decode(&data),
            Self::Utf16Be => {
                let owned = data.to_vec();
                encodings::utf16be::decode(owned)
            }
        })
    }
}

/// The `TextEncoder`[mdn] class represents an encoder for a specific method, that is
/// a specific character encoding, like `utf-8`.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder
#[derive(Debug, Default, Clone, JsData, Trace, Finalize)]
pub enum TextEncoder {
    /// Encode UTF-8 strings into buffers.
    #[default]
    Utf8,
    /// Encode UTF-16 strings (little endian) into buffers.
    Utf16Le,
    /// Encode UTF-16 strings (big endian) into buffers.
    Utf16Be,
}

#[boa_class]
impl TextEncoder {
    /// The [`TextEncoder()`][mdn] constructor returns a newly created `TextEncoder` object.
    ///
    /// # Errors
    /// This will return an error if the encoding or options are invalid or unsupported.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder/TextEncoder
    #[boa(constructor)]
    pub fn constructor(encoding: Option<JsString>, _options: Option<JsObject>) -> JsResult<Self> {
        let Some(encoding) = encoding else {
            return Ok(Self::default());
        };

        match encoding.to_std_string_lossy().as_str() {
            "utf-8" => Ok(Self::Utf8),
            // Default encoding is Little Endian.
            "utf-16" | "utf-16le" => Ok(Self::Utf16Le),
            "utf-16be" => Ok(Self::Utf16Be),
            e => Err(js_error!(RangeError: "The given encoding '{}' is not supported.", e)),
        }
    }

    /// The [`TextEncoder.encoding`][mdn] read-only property returns a string containing
    /// the name of the encoding algorithm used by the specific encoder.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder/encoding
    #[boa(getter)]
    #[must_use]
    fn encoding(&self) -> JsString {
        match self {
            Self::Utf8 => js_string!("utf-8"),
            Self::Utf16Le => js_string!("utf-16le"),
            Self::Utf16Be => js_string!("utf-16be"),
        }
    }

    /// The [`TextEncoder.encode()`][mdn] method takes a string as input, and returns
    /// a `Uint8Array` containing the string encoded using UTF-8.
    ///
    /// # Errors
    /// This will error if there is an issue creating the `Uint8Array` or encoding
    /// the string itself.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder/encode
    pub fn encode(&self, text: Option<JsString>, context: &mut Context) -> JsResult<JsUint8Array> {
        let Some(text) = text else {
            return JsUint8Array::from_iter([], context);
        };

        let vec = match self {
            Self::Utf8 => encodings::utf8::encode(&text),
            Self::Utf16Le => encodings::utf16le::encode(&text),
            Self::Utf16Be => encodings::utf16be::encode(&text),
        };
        JsUint8Array::from_iter(vec, context)
    }
}

/// JavaScript module containing the text encoding/decoding classes.
#[boa_module]
pub mod js_module {
    type TextDecoder = super::TextDecoder;
    type TextEncoder = super::TextEncoder;
}

/// Register both `TextDecoder` and `TextEncoder` classes into the realm/context.
///
/// # Errors
/// This will error if the context or realm cannot register the class.
pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
    js_module::boa_register(realm, context)
}
