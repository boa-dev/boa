//! Base64 utility methods (`atob` and `btoa`).
//!
//! See <https://html.spec.whatwg.org/multipage/webappapis.html#atob>.
#![allow(clippy::needless_pass_by_value)]

use boa_engine::realm::Realm;
use boa_engine::{Context, JsResult, boa_module};

#[cfg(test)]
mod tests;

/// The Base64 alphabet used by `btoa`/`atob`.
const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Decodes a single Base64 character to its 6-bit value.
fn base64_decode_char(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// JavaScript module containing the `atob` and `btoa` functions.
#[boa_module]
pub mod js_module {
    use super::{BASE64_CHARS, base64_decode_char};
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
        // Per spec: if any code point > 0xFF, throw InvalidCharacterError.
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

        let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);

        for chunk in bytes.chunks(3) {
            let b0 = u32::from(chunk[0]);
            let b1 = u32::from(*chunk.get(1).unwrap_or(&0));
            let b2 = u32::from(*chunk.get(2).unwrap_or(&0));
            let n = (b0 << 16) | (b1 << 8) | b2;

            result.push(char::from(BASE64_CHARS[((n >> 18) & 0x3F) as usize]));
            result.push(char::from(BASE64_CHARS[((n >> 12) & 0x3F) as usize]));

            if chunk.len() > 1 {
                result.push(char::from(BASE64_CHARS[((n >> 6) & 0x3F) as usize]));
            } else {
                result.push('=');
            }

            if chunk.len() > 2 {
                result.push(char::from(BASE64_CHARS[(n & 0x3F) as usize]));
            } else {
                result.push('=');
            }
        }

        Ok(result)
    }

    /// The [`atob()`][mdn] method decodes a string of data which has been
    /// encoded using Base64 encoding.
    ///
    /// # Errors
    /// Throws a `DOMException` (`InvalidCharacterError`) if the input is
    /// not valid Base64.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Window/atob
    #[allow(clippy::cast_possible_truncation, clippy::many_single_char_names)]
    pub fn atob(data: String) -> JsResult<String> {
        // Step 1: Remove all ASCII whitespace from data.
        let cleaned: Vec<u8> = data
            .bytes()
            .filter(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\x0C' | b'\r'))
            .collect();

        // Step 2: If length is a multiple of 4, remove 1 or 2 trailing '=' chars.
        let cleaned = if cleaned.len().is_multiple_of(4) {
            let mut end = cleaned.len();
            if end > 0 && cleaned[end - 1] == b'=' {
                end -= 1;
            }
            if end > 0 && cleaned[end - 1] == b'=' {
                end -= 1;
            }
            &cleaned[..end]
        } else {
            &cleaned[..]
        };

        // Step 3: If length % 4 == 1, throw.
        if cleaned.len() % 4 == 1 {
            return Err(js_error!(
                "InvalidCharacterError: The string to be decoded is not correctly encoded."
            ));
        }

        // Validate all remaining chars are in the Base64 alphabet.
        for &b in cleaned {
            if base64_decode_char(b).is_none() {
                return Err(js_error!(
                    "InvalidCharacterError: The string to be decoded is not correctly encoded."
                ));
            }
        }

        // Decode: each group of 4 Base64 chars yields 3 bytes.
        let mut output = Vec::with_capacity(cleaned.len() * 3 / 4);

        for chunk in cleaned.chunks(4) {
            let a = u32::from(base64_decode_char(chunk[0]).unwrap_or(0));
            let b = u32::from(
                chunk
                    .get(1)
                    .and_then(|&c| base64_decode_char(c))
                    .unwrap_or(0),
            );
            let c = u32::from(
                chunk
                    .get(2)
                    .and_then(|&c| base64_decode_char(c))
                    .unwrap_or(0),
            );
            let d = u32::from(
                chunk
                    .get(3)
                    .and_then(|&c| base64_decode_char(c))
                    .unwrap_or(0),
            );

            let n = (a << 18) | (b << 12) | (c << 6) | d;

            // Truncation is intentional: we extract individual bytes from a 24-bit value.
            output.push((n >> 16) as u8);
            if chunk.len() > 2 {
                output.push((n >> 8) as u8);
            }
            if chunk.len() > 3 {
                output.push(n as u8);
            }
        }

        // Per spec: result is a string where each byte becomes a code point.
        Ok(output.into_iter().map(char::from).collect())
    }
}

/// Register the `atob` and `btoa` functions in the global context.
///
/// # Errors
/// Returns an error if the functions cannot be registered.
pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
    js_module::boa_register(realm, context)
}
