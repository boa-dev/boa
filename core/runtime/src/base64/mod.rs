//! Base64 utility methods (`atob` and `btoa`).
//!
//! See <https://html.spec.whatwg.org/multipage/webappapis.html#atob>.
#![allow(clippy::needless_pass_by_value)]

use boa_engine::realm::Realm;
use boa_engine::{Context, JsResult, boa_module};

#[cfg(test)]
mod tests;

/// A forgiving Base64 engine that accepts input with or without padding,
/// matching the [forgiving-base64 decode](https://infra.spec.whatwg.org/#forgiving-base64-decode)
/// algorithm used by `atob`.
const FORGIVING: base64::engine::GeneralPurpose = base64::engine::GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    base64::engine::general_purpose::GeneralPurposeConfig::new()
        .with_decode_padding_mode(base64::engine::DecodePaddingMode::Indifferent),
);

/// JavaScript module containing the `atob` and `btoa` functions.
#[boa_module]
pub mod js_module {
    use super::FORGIVING;
    use base64::Engine as _;
    use boa_engine::{JsResult, js_error};

    /// The [`btoa()`][mdn] method creates a Base64-encoded ASCII string from
    /// a binary string (i.e., a string in which each character is treated as
    /// a byte of binary data).
    ///
    /// # Errors
    /// Throws a `DOMException` (`InvalidCharacterError`) if the string
    /// contains any character whose code point is greater than `0xFF`.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Window/btoa
    #[allow(clippy::cast_possible_truncation)]
    pub fn btoa(data: String) -> JsResult<String> {
        let bytes: Vec<u8> = data
            .chars()
            .map(|c| {
                let cp = c as u32;
                if cp > 0xFF {
                    Err(js_error!("InvalidCharacterError: The string to be encoded contains characters outside of the Latin1 range."))
                } else {
                    Ok(cp as u8)
                }
            })
            .collect::<JsResult<Vec<u8>>>()?;

        Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
    }

    /// The [`atob()`][mdn] method decodes a string of data which has been
    /// encoded using Base64 encoding.
    ///
    /// # Errors
    /// Throws a `DOMException` (`InvalidCharacterError`) if the input is
    /// not valid Base64.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Window/atob
    pub fn atob(data: String) -> JsResult<String> {
        let cleaned: String = data
            .chars()
            .filter(|c| !matches!(c, ' ' | '\t' | '\n' | '\x0C' | '\r'))
            .collect();

        let bytes = FORGIVING.decode(cleaned.as_bytes()).map_err(|_| {
            js_error!("InvalidCharacterError: The string to be decoded is not correctly encoded.")
        })?;

        Ok(bytes.into_iter().map(char::from).collect())
    }
}

/// Register the `atob` and `btoa` functions in the global context.
///
/// # Errors
/// Returns an error if the functions cannot be registered.
pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
    js_module::boa_register(realm, context)
}
