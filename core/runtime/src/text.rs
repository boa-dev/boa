//! Module implementing JavaScript classes to handle text encoding and decoding.
//!
//! See <https://developer.mozilla.org/en-US/docs/Web/API/Encoding_API> for more information.

use boa_engine::object::builtins::JsUint8Array;
use boa_engine::string::CodePoint;
use boa_engine::{
    js_string, Context, Finalize, JsData, JsNativeError, JsObject, JsResult, JsString, Trace,
};
use boa_interop::js_class;

#[cfg(test)]
mod tests;

/// The `TextDecoder`[mdn] class represents an encoder for a specific method, that is
/// a specific character encoding, like `utf-8`.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder
#[derive(Debug, Clone, JsData, Trace, Finalize)]
pub struct TextDecoder;

impl TextDecoder {
    /// Register the `TextDecoder` class into the realm.
    ///
    /// # Errors
    /// This will error if the context or realm cannot register the class.
    pub fn register(context: &mut Context) -> JsResult<()> {
        context.register_global_class::<Self>()?;
        Ok(())
    }

    /// The `decode()` method of the `TextDecoder` interface returns a `JsString` containing
    /// the given `Uint8Array` decoded in the specific method. This will replace any
    /// invalid characters with the Unicode replacement character.
    pub fn decode(text: &JsUint8Array, context: &mut Context) -> JsString {
        let buffer = text.iter(context).collect::<Vec<u8>>();
        let string = String::from_utf8_lossy(&buffer);
        JsString::from(string.as_ref())
    }
}

js_class! {
    class TextDecoder {
        property encoding {
            fn get() -> JsString {
                js_string!("utf-8")
            }
        }

        // Creates a new `TextEncoder` object. Encoding is optional but MUST BE
        // "utf-8" if specified. Options is ignored.
        constructor(encoding: Option<JsString>, _options: Option<JsObject>) {
            if let Some(e) = encoding {
                if e != js_string!("utf-8") {
                    return Err(JsNativeError::typ().with_message("Only utf-8 encoding is supported").into());
                }
            }
            Ok(TextDecoder)
        }

        fn decode(array: JsUint8Array, context: &mut Context) -> JsString {
            TextDecoder::decode(&array, context)
        }
    }
}

/// The `TextEncoder`[mdn] class represents an encoder for a specific method, that is
/// a specific character encoding, like `utf-8`.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder
#[derive(Debug, Clone, JsData, Trace, Finalize)]
pub struct TextEncoder;

impl TextEncoder {
    /// Register the `TextEncoder` class into the realm.
    ///
    /// # Errors
    /// This will error if the context or realm cannot register the class.
    pub fn register(context: &mut Context) -> JsResult<()> {
        context.register_global_class::<Self>()?;
        Ok(())
    }

    /// The `encode()` method of the `TextEncoder` interface returns a `Uint8Array` containing
    /// the given string encoded in the specific method.
    ///
    /// # Errors
    /// This will error if there is an issue creating the `Uint8Array`.
    pub fn encode(text: &JsString, context: &mut Context) -> JsResult<JsUint8Array> {
        // TODO: move this logic to JsString.
        JsUint8Array::from_iter(
            text.code_points().flat_map(|s| match s {
                CodePoint::Unicode(c) => c.to_string().as_bytes().to_vec(),
                CodePoint::UnpairedSurrogate(_) => "\u{FFFD}".as_bytes().to_vec(),
            }),
            context,
        )
    }
}

js_class! {
    class TextEncoder {
        property encoding {
            fn get() -> JsString {
                js_string!("utf-8")
            }
        }

        // Creates a new `TextEncoder` object. Encoding is optional but MUST BE
        // "utf-8" if specified. Options is ignored.
        constructor(encoding: Option<JsString>, _options: Option<JsObject>) {
            if let Some(e) = encoding {
                if e != js_string!("utf-8") {
                    return Err(JsNativeError::typ().with_message("Only utf-8 encoding is supported").into());
                }
            }
            Ok(TextEncoder)
        }

        fn encode(text: JsString, context: &mut Context) -> JsResult<JsUint8Array> {
            TextEncoder::encode(&text, context)
        }
    }
}
