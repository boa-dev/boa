//! Module implementing JavaScript classes to handle text encoding and decoding.
//!
//! See <https://developer.mozilla.org/en-US/docs/Web/API/Encoding_API> for more information.

use boa_engine::object::builtins::{JsArrayBuffer, JsDataView, JsTypedArray, JsUint8Array};
use boa_engine::realm::Realm;
use boa_engine::value::TryFromJs;
use boa_engine::{
    Context, Finalize, JsData, JsObject, JsResult, JsString, JsValue, Trace, boa_class, boa_module,
    js_error, js_string,
};

#[cfg(test)]
mod tests;

mod encodings;

/// Options for the [`TextDecoder`] constructor.
#[derive(Debug, Default, Clone, Copy, TryFromJs)]
pub struct TextDecoderOptions {
    #[boa(rename = "ignoreBOM")]
    ignore_bom: Option<bool>,
}

/// The character encoding used by [`TextDecoder`].
#[derive(Debug, Default, Clone, Copy)]
pub enum Encoding {
    /// UTF-8 encoding.
    #[default]
    Utf8,
    /// UTF-16 little endian encoding.
    Utf16Le,
    /// UTF-16 big endian encoding.
    Utf16Be,
}

/// The [`TextDecoder`][mdn] class represents an encoder for a specific method, that is
/// a specific character encoding, like `utf-8`.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder
#[derive(Debug, Default, Clone, JsData, Trace, Finalize)]
pub struct TextDecoder {
    #[unsafe_ignore_trace]
    encoding: Encoding,
    #[unsafe_ignore_trace]
    ignore_bom: bool,
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
    pub fn constructor(
        encoding: Option<JsString>,
        options: Option<TextDecoderOptions>,
    ) -> JsResult<Self> {
        let ignore_bom = options.and_then(|o| o.ignore_bom).unwrap_or(false);

        let encoding = match encoding {
            Some(enc) => match enc.to_std_string_lossy().as_str() {
                "utf-8" => Encoding::Utf8,
                // Default encoding is Little Endian.
                "utf-16" | "utf-16le" => Encoding::Utf16Le,
                "utf-16be" => Encoding::Utf16Be,
                e => {
                    return Err(
                        js_error!(RangeError: "The given encoding '{}' is not supported.", e),
                    );
                }
            },
            None => Encoding::default(),
        };

        Ok(Self {
            encoding,
            ignore_bom,
        })
    }

    /// The [`TextDecoder.encoding`][mdn] read-only property returns a string containing
    /// the name of the character encoding that this decoder will use.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/encoding
    #[boa(getter)]
    #[must_use]
    pub fn encoding(&self) -> JsString {
        match self.encoding {
            Encoding::Utf8 => js_string!("utf-8"),
            Encoding::Utf16Le => js_string!("utf-16le"),
            Encoding::Utf16Be => js_string!("utf-16be"),
        }
    }

    /// The [`TextDecoder.ignoreBOM`][mdn] read-only property returns a `bool` indicating
    /// whether the BOM (byte order mark) is ignored.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/ignoreBOM
    #[boa(getter)]
    #[boa(rename = "ignoreBOM")]
    #[must_use]
    pub fn ignore_bom(&self) -> bool {
        self.ignore_bom
    }

    /// The [`TextDecoder.decode()`][mdn] method returns a string containing text decoded from the
    /// buffer passed as a parameter.
    ///
    /// `buffer` can be an `ArrayBuffer`, a `TypedArray` or a `DataView`.
    ///
    /// # Errors
    /// Any error that arises during decoding the specific encoding.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/decode
    pub fn decode(&self, buffer: JsValue, context: &mut Context) -> JsResult<JsString> {
        let mut range = None;
        let array_buffer = if let Ok(array_buffer) = JsArrayBuffer::try_from_js(&buffer, context) {
            array_buffer
        } else if let Ok(typed_array) = JsTypedArray::try_from_js(&buffer, context) {
            let Some(obj) = typed_array.buffer(context)?.as_object() else {
                return Err(js_error!(TypeError: "Invalid buffer backing TypedArray."));
            };

            let offset = typed_array.byte_offset(context)?;
            let length = typed_array.byte_length(context)?;

            range = Some(offset..offset + length);

            JsArrayBuffer::from_object(obj)?
        } else if let Ok(data_view) = JsDataView::try_from_js(&buffer, context) {
            let Some(obj) = data_view.buffer(context)?.as_object() else {
                return Err(js_error!(TypeError: "Invalid buffer backing DataView."));
            };

            JsArrayBuffer::from_object(obj)?
        } else {
            return Err(js_error!(
                TypeError: "Argument 1 must be an ArrayBuffer, TypedArray or DataView."
            ));
        };

        let strip_bom = !self.ignore_bom;

        let Some(full_data) = array_buffer.data() else {
            return Err(js_error!(TypeError: "cannot decode a detached ArrayBuffer"));
        };

        let data: &[u8] = if let Some(range) = range {
            full_data.get(range).ok_or_else(
                // We do not say invalid range here, as both subarray(10, 5) and subarray("a", "b")
                // are valid JS, it would just an empty array. If this error occurs, it most likely means something else
                // is wrong
                || js_error!(RangeError: "The range for the underlying ArrayBuffer can not be accessed."),
            )?
        } else {
            &full_data
        };

        Ok(match self.encoding {
            Encoding::Utf8 => encodings::utf8::decode(&data, strip_bom),
            Encoding::Utf16Le => encodings::utf16le::decode(&data, strip_bom),
            Encoding::Utf16Be => {
                let owned = data.to_vec();
                encodings::utf16be::decode(owned, strip_bom)
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
