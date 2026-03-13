//! Implementation of `Uint8Array` Base64 and Hex encoding/decoding methods.
//!
//! Spec: <https://tc39.es/proposal-arraybuffer-base64/>

use std::sync::atomic::Ordering;

use crate::{
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
    builtins::typed_array::{TypedArray, TypedArrayKind},
    js_string,
    object::{
        JsObject,
        builtins::{AlignedVec, JsArrayBuffer},
    },
};

// ===== Base64 Tables =====

/// Decode a single base64 character to its 6-bit value.
fn base64_decode_char(c: u8, use_url: bool) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' if !use_url => Some(62),
        b'/' if !use_url => Some(63),
        b'-' if use_url => Some(62),
        b'_' if use_url => Some(63),
        _ => None,
    }
}

/// Is this an ASCII whitespace character per the spec?
fn is_ascii_whitespace(c: u8) -> bool {
    matches!(c, 0x09 | 0x0A | 0x0C | 0x0D | 0x20)
}

/// Options for Base64 operations.
#[derive(Debug, Clone, Copy)]
enum Alphabet {
    Base64,
    Base64Url,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LastChunkHandling {
    Loose,
    Strict,
    StopBeforePartial,
}

/// Parse the `alphabet` option from an options object.
/// Per spec, the value must be a string — not coerced via toString.
fn get_alphabet(options: &JsObject, context: &mut Context) -> JsResult<Alphabet> {
    let val = options.get(js_string!("alphabet"), context)?;
    if val.is_undefined() {
        return Ok(Alphabet::Base64);
    }
    let s = val
        .as_string()
        .ok_or_else(|| JsNativeError::typ().with_message("alphabet must be a string"))?;
    match s.to_std_string_escaped().as_str() {
        "base64" => Ok(Alphabet::Base64),
        "base64url" => Ok(Alphabet::Base64Url),
        other => Err(JsNativeError::typ()
            .with_message(format!("Invalid alphabet option: '{other}'"))
            .into()),
    }
}

/// Parse the `lastChunkHandling` option from an options object.
/// Per spec, the value must be a string — not coerced via toString.
fn get_last_chunk_handling(
    options: &JsObject,
    context: &mut Context,
) -> JsResult<LastChunkHandling> {
    let val = options.get(js_string!("lastChunkHandling"), context)?;
    if val.is_undefined() {
        return Ok(LastChunkHandling::Loose);
    }
    let s = val
        .as_string()
        .ok_or_else(|| JsNativeError::typ().with_message("lastChunkHandling must be a string"))?;
    match s.to_std_string_escaped().as_str() {
        "loose" => Ok(LastChunkHandling::Loose),
        "strict" => Ok(LastChunkHandling::Strict),
        "stop-before-partial" => Ok(LastChunkHandling::StopBeforePartial),
        other => Err(JsNativeError::typ()
            .with_message(format!("Invalid lastChunkHandling option: '{other}'"))
            .into()),
    }
}

/// Parse the `omitPadding` option from an options object.
fn get_omit_padding(options: &JsObject, context: &mut Context) -> JsResult<bool> {
    let val = options.get(js_string!("omitPadding"), context)?;
    if val.is_undefined() {
        return Ok(false);
    }
    Ok(val.to_boolean())
}

// ===== Core Base64 Decode =====

/// Result of a `FromBase64` operation, following the spec's record structure.
struct FromBase64Result {
    read: usize,
    bytes: Vec<u8>,
    error: Option<String>,
}

/// Spec: `FromBase64 ( string, alphabet, lastChunkHandling [ , maxLength ] )`
///
/// This carefully follows the spec algorithm step-by-step to handle all edge cases
/// correctly, including partial chunks, padding validation, and maxLength truncation.
fn spec_from_base64(
    input: &str,
    use_url: bool,
    last_chunk: LastChunkHandling,
    max_length: Option<usize>,
) -> FromBase64Result {
    let max_length = max_length.unwrap_or(usize::MAX);

    // Step 3: If maxLength is 0, return immediately.
    if max_length == 0 {
        return FromBase64Result {
            read: 0,
            bytes: Vec::new(),
            error: None,
        };
    }

    let input_bytes = input.as_bytes();
    let length = input_bytes.len();

    // Process character-by-character, tracking the `read` position
    // across chunk boundaries.
    let mut bytes = Vec::new();
    let mut read = 0;
    let mut chunk = [0u8; 4];
    let mut chunk_len: usize = 0;

    let mut i = 0;
    while i < length {
        let c = input_bytes[i];

        if is_ascii_whitespace(c) {
            i += 1;
            continue;
        }

        // If it's a '=', handle padding
        if c == b'=' {
            // Per spec 10.g: If chunk has fewer than 2 chars, padding is invalid
            if chunk_len < 2 {
                return FromBase64Result {
                    read,
                    bytes,
                    error: Some("Unexpected padding".to_string()),
                };
            }

            // Handle based on chunk_len
            if chunk_len == 2 {
                // Need exactly '=='
                // Skip whitespace to find second '='
                let mut j = i + 1;
                while j < length && is_ascii_whitespace(input_bytes[j]) {
                    j += 1;
                }
                if j >= length || input_bytes[j] != b'=' {
                    // Only one '=' with chunk_len 2 → partial padding
                    // This is a partial chunk. For stop-before-partial, stop before it.
                    if last_chunk == LastChunkHandling::StopBeforePartial {
                        return FromBase64Result {
                            read,
                            bytes,
                            error: None,
                        };
                    }
                    return FromBase64Result {
                        read,
                        bytes,
                        error: Some("Expected second padding character".to_string()),
                    };
                }
                // We have '=='
                // Check strict mode: non-zero padding bits
                if last_chunk == LastChunkHandling::Strict && (chunk[1] & 0x0F) != 0 {
                    return FromBase64Result {
                        read,
                        bytes,
                        error: Some("Non-zero padding bits".to_string()),
                    };
                }

                // Decode 1 byte
                let val = (u32::from(chunk[0]) << 18) | (u32::from(chunk[1]) << 12);

                // Check for trailing non-whitespace after '=='
                let pad_end = j + 1;
                let mut k = pad_end;
                while k < length && is_ascii_whitespace(input_bytes[k]) {
                    k += 1;
                }

                // If trailing data exists, error WITHOUT including this chunk's bytes
                if k < length {
                    return FromBase64Result {
                        read,
                        bytes,
                        error: Some("Extra data after padding".to_string()),
                    };
                }

                // Only push bytes after confirming no trailing data
                if bytes.len() < max_length {
                    bytes.push((val >> 16) as u8);
                }

                read = pad_end;
                chunk_len = 0;
                i = pad_end;
                continue;
            }

            if chunk_len == 3 {
                // Need exactly '='
                // Check strict mode: non-zero padding bits
                if last_chunk == LastChunkHandling::Strict && (chunk[2] & 0x03) != 0 {
                    return FromBase64Result {
                        read,
                        bytes,
                        error: Some("Non-zero padding bits".to_string()),
                    };
                }

                let val = (u32::from(chunk[0]) << 18)
                    | (u32::from(chunk[1]) << 12)
                    | (u32::from(chunk[2]) << 6);

                let pad_end = i + 1;

                // Check for trailing non-whitespace after '='
                let mut k = pad_end;
                while k < length && is_ascii_whitespace(input_bytes[k]) {
                    k += 1;
                }

                // If trailing data exists, error WITHOUT including this chunk's bytes
                if k < length {
                    return FromBase64Result {
                        read,
                        bytes,
                        error: Some("Extra data after padding".to_string()),
                    };
                }

                // Only push bytes after confirming no trailing data
                // All-or-nothing: this padded chunk produces 2 bytes
                let remaining = max_length.saturating_sub(bytes.len());
                if remaining < 2 {
                    return FromBase64Result {
                        read,
                        bytes,
                        error: None,
                    };
                }
                bytes.push((val >> 16) as u8);
                bytes.push((val >> 8) as u8);

                read = pad_end;
                chunk_len = 0;
                i = pad_end;
                continue;
            }

            // chunk_len is 0 or 1: padding is invalid (caught above with < 2 check)
            unreachable!();
        }

        // Regular base64 character
        let Some(val) = base64_decode_char(c, use_url) else {
            return FromBase64Result {
                read,
                bytes,
                error: Some("Invalid character".to_string()),
            };
        };

        chunk[chunk_len] = val;
        chunk_len += 1;
        i += 1;

        if chunk_len == 4 {
            let v = (u32::from(chunk[0]) << 18)
                | (u32::from(chunk[1]) << 12)
                | (u32::from(chunk[2]) << 6)
                | u32::from(chunk[3]);

            let remaining = max_length.saturating_sub(bytes.len());
            // All-or-nothing: if all 3 bytes don't fit, stop before this chunk
            if remaining < 3 {
                return FromBase64Result {
                    read,
                    bytes,
                    error: None,
                };
            }

            bytes.push((v >> 16) as u8);
            bytes.push((v >> 8) as u8);
            bytes.push(v as u8);

            read = i;
            chunk_len = 0;

            if bytes.len() >= max_length {
                return FromBase64Result {
                    read,
                    bytes,
                    error: None,
                };
            }
        }
    }

    // Handle remaining partial chunk at end of input
    if chunk_len > 0 {
        match last_chunk {
            LastChunkHandling::StopBeforePartial => {
                // Don't process partial chunk, read stays at last complete position
            }
            LastChunkHandling::Strict => {
                return FromBase64Result {
                    read,
                    bytes,
                    error: Some("Missing padding in strict mode".to_string()),
                };
            }
            LastChunkHandling::Loose => {
                if chunk_len == 1 {
                    return FromBase64Result {
                        read,
                        bytes,
                        error: Some("Single character chunk is invalid".to_string()),
                    };
                }
                if chunk_len == 2 {
                    if bytes.len() >= max_length {
                        return FromBase64Result {
                            read,
                            bytes,
                            error: None,
                        };
                    }
                    let val = (u32::from(chunk[0]) << 18) | (u32::from(chunk[1]) << 12);
                    bytes.push((val >> 16) as u8);
                    read = i;
                } else {
                    // chunk_len == 3, produces 2 bytes — all-or-nothing
                    let remaining = max_length.saturating_sub(bytes.len());
                    if remaining < 2 {
                        return FromBase64Result {
                            read,
                            bytes,
                            error: None,
                        };
                    }
                    let val = (u32::from(chunk[0]) << 18)
                        | (u32::from(chunk[1]) << 12)
                        | (u32::from(chunk[2]) << 6);
                    bytes.push((val >> 16) as u8);
                    bytes.push((val >> 8) as u8);
                    read = i;
                }
            }
        }
    }

    FromBase64Result {
        read,
        bytes,
        error: None,
    }
}

// ===== Core Base64 Encode =====

/// Standard Base64 alphabet (RFC 4648 §4)
const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// URL-safe Base64 alphabet (RFC 4648 §5)
const BASE64URL_CHARS: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn base64_encode_impl(data: &[u8], use_url: bool, omit_padding: bool) -> String {
    let table = if use_url {
        BASE64URL_CHARS
    } else {
        BASE64_CHARS
    };
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        match chunk.len() {
            3 => {
                let n =
                    (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
                result.push(char::from(table[((n >> 18) & 0x3F) as usize]));
                result.push(char::from(table[((n >> 12) & 0x3F) as usize]));
                result.push(char::from(table[((n >> 6) & 0x3F) as usize]));
                result.push(char::from(table[(n & 0x3F) as usize]));
            }
            2 => {
                let n = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8);
                result.push(char::from(table[((n >> 18) & 0x3F) as usize]));
                result.push(char::from(table[((n >> 12) & 0x3F) as usize]));
                result.push(char::from(table[((n >> 6) & 0x3F) as usize]));
                if !omit_padding {
                    result.push('=');
                }
            }
            1 => {
                let n = u32::from(chunk[0]) << 16;
                result.push(char::from(table[((n >> 18) & 0x3F) as usize]));
                result.push(char::from(table[((n >> 12) & 0x3F) as usize]));
                if !omit_padding {
                    result.push('=');
                    result.push('=');
                }
            }
            _ => unreachable!(),
        }
    }

    result
}

// ===== Core Hex Decode/Encode =====

/// Decode a single hex character to its 4-bit value.
fn hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Result of a `FromHex` operation.
struct FromHexResult {
    read: usize,
    bytes: Vec<u8>,
    error: Option<String>,
}

/// Core `FromHex` algorithm per spec.
fn spec_from_hex(input: &str, max_length: Option<usize>) -> FromHexResult {
    let input_bytes = input.as_bytes();
    let length = input_bytes.len();
    let max_length = max_length.unwrap_or(usize::MAX);

    // Step: If length is odd, return error.
    if !length.is_multiple_of(2) {
        return FromHexResult {
            read: 0,
            bytes: Vec::new(),
            error: Some("Hex string must have an even number of characters".to_string()),
        };
    }

    let mut output = Vec::with_capacity(length / 2);
    let mut i = 0;

    while i < length {
        if output.len() >= max_length {
            return FromHexResult {
                read: i,
                bytes: output,
                error: None,
            };
        }

        let Some(hi) = hex_digit(input_bytes[i]) else {
            return FromHexResult {
                read: i,
                bytes: output,
                error: Some(format!("Invalid hex character at position {i}")),
            };
        };
        let Some(lo) = hex_digit(input_bytes[i + 1]) else {
            return FromHexResult {
                read: i,
                bytes: output,
                error: Some(format!("Invalid hex character at position {}", i + 1)),
            };
        };

        output.push((hi << 4) | lo);
        i += 2;
    }

    FromHexResult {
        read: i,
        bytes: output,
        error: None,
    }
}

/// Encode bytes as a lowercase hex string.
fn hex_encode_impl(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len() * 2);
    for &b in data {
        result.push(char::from(b"0123456789abcdef"[(b >> 4) as usize]));
        result.push(char::from(b"0123456789abcdef"[(b & 0x0F) as usize]));
    }
    result
}

// ===== Helpers =====

/// Validate that `this` is a `Uint8Array`, return the typed object.
fn validate_uint8array(this: &JsValue) -> JsResult<JsObject<TypedArray>> {
    let object = this.as_object();
    let ta = object
        .as_ref()
        .and_then(|o| o.clone().downcast::<TypedArray>().ok())
        .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a Uint8Array"))?;

    if ta.borrow().data().kind() != TypedArrayKind::Uint8 {
        return Err(JsNativeError::typ()
            .with_message("`this` is not a Uint8Array")
            .into());
    }

    Ok(ta)
}

/// Get the raw bytes from a `Uint8Array`.
fn get_uint8_bytes(ta: &JsObject<TypedArray>) -> JsResult<Vec<u8>> {
    let ta_borrow = ta.borrow();
    let data = ta_borrow.data();

    let buf = data.viewed_array_buffer();
    let buffer = buf.as_buffer();
    let Some(slice) = buffer.bytes(Ordering::SeqCst) else {
        return Err(JsNativeError::typ()
            .with_message("ArrayBuffer is detached")
            .into());
    };

    let byte_offset = data.byte_offset() as usize;
    let buf_len = slice.len();
    let array_len = data.array_length(buf_len) as usize;

    Ok(slice
        .subslice(byte_offset..byte_offset + array_len)
        .to_vec())
}

/// Get the length of a `Uint8Array`.
fn get_uint8_length(ta: &JsObject<TypedArray>) -> JsResult<usize> {
    let ta_borrow = ta.borrow();
    let data = ta_borrow.data();
    let buf = data.viewed_array_buffer();
    let buffer = buf.as_buffer();
    let Some(slice) = buffer.bytes(Ordering::SeqCst) else {
        return Err(JsNativeError::typ()
            .with_message("ArrayBuffer is detached")
            .into());
    };
    Ok(data.array_length(slice.len()) as usize)
}

/// Write decoded bytes into a `Uint8Array`'s buffer.
fn write_uint8_bytes(ta: &JsObject<TypedArray>, decoded: &[u8]) -> JsResult<()> {
    let ta_borrow = ta.borrow();
    let data = ta_borrow.data();
    let buf = data.viewed_array_buffer();
    let mut buffer = buf.as_buffer_mut();
    let Some(mut raw) = buffer.bytes(Ordering::SeqCst) else {
        return Err(JsNativeError::typ()
            .with_message("ArrayBuffer is detached")
            .into());
    };
    let byte_offset = data.byte_offset() as usize;
    let mut target = raw.subslice_mut(byte_offset..);
    for (i, &b) in decoded.iter().enumerate() {
        // SAFETY: We have validated that `decoded.len()` fits within
        // the typed array's byte length before calling this function.
        unsafe {
            target.subslice_mut(i..i + 1).set_value(
                crate::builtins::typed_array::TypedArrayElement::Uint8(b),
                Ordering::Relaxed,
            );
        }
    }
    Ok(())
}

// ===== Shared Helpers =====

/// Parse base64 options (alphabet + lastChunkHandling) from the options argument.
fn parse_base64_options(
    opts: &JsValue,
    context: &mut Context,
) -> JsResult<(bool, LastChunkHandling)> {
    if opts.is_undefined() {
        return Ok((false, LastChunkHandling::Loose));
    }
    let opts_obj = opts
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Options must be an object"))?;
    let alphabet = get_alphabet(&opts_obj, context)?;
    let lch = get_last_chunk_handling(&opts_obj, context)?;
    Ok((matches!(alphabet, Alphabet::Base64Url), lch))
}

/// Create a new `Uint8Array` from decoded bytes.
fn create_uint8array_from_bytes(bytes: Vec<u8>, context: &mut Context) -> JsResult<JsValue> {
    let data = AlignedVec::from_iter(0, bytes);
    let buf = JsArrayBuffer::from_byte_block(data, context)?;
    let uint8_constructor = context
        .intrinsics()
        .constructors()
        .typed_uint8_array()
        .constructor();
    uint8_constructor
        .construct(&[buf.into()], Some(&uint8_constructor), context)
        .map(JsValue::from)
}

// ===== JS Method Implementations =====

/// `Uint8Array.fromBase64 ( string [ , options ] )`
pub(super) fn from_base64(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let input = args
        .get_or_undefined(0)
        .as_string()
        .ok_or_else(|| JsNativeError::typ().with_message("First argument must be a string"))?
        .to_std_string_escaped();

    let (use_url, last_chunk) = parse_base64_options(args.get_or_undefined(1), context)?;

    let result = spec_from_base64(&input, use_url, last_chunk, None);

    if let Some(err) = result.error {
        return Err(JsNativeError::syntax().with_message(err).into());
    }

    create_uint8array_from_bytes(result.bytes, context)
}

/// `Uint8Array.fromHex ( string )`
pub(super) fn from_hex(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let input = args
        .get_or_undefined(0)
        .as_string()
        .ok_or_else(|| JsNativeError::typ().with_message("First argument must be a string"))?
        .to_std_string_escaped();

    let result = spec_from_hex(&input, None);

    if let Some(err) = result.error {
        return Err(JsNativeError::syntax().with_message(err).into());
    }

    create_uint8array_from_bytes(result.bytes, context)
}

/// `Uint8Array.prototype.toBase64 ( [ options ] )`
pub(super) fn to_base64(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ta = validate_uint8array(this)?;

    let opts = args.get_or_undefined(0);
    let (use_url, omit_padding) = if opts.is_undefined() {
        (false, false)
    } else {
        let opts_obj = opts
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("Options must be an object"))?;
        let alphabet = get_alphabet(&opts_obj, context)?;
        let omit = get_omit_padding(&opts_obj, context)?;
        (matches!(alphabet, Alphabet::Base64Url), omit)
    };
    // Note: toBase64 has `omitPadding` so we can't reuse `parse_base64_options`.

    let raw_bytes = get_uint8_bytes(&ta)?;
    let encoded = base64_encode_impl(&raw_bytes, use_url, omit_padding);
    Ok(JsString::from(encoded).into())
}

/// `Uint8Array.prototype.toHex ( )`
pub(super) fn to_hex(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let ta = validate_uint8array(this)?;
    let raw_bytes = get_uint8_bytes(&ta)?;
    let encoded = hex_encode_impl(&raw_bytes);
    Ok(JsString::from(encoded).into())
}

/// `Uint8Array.prototype.setFromBase64 ( string [ , options ] )`
pub(super) fn set_from_base64(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ta = validate_uint8array(this)?;

    let input = args
        .get_or_undefined(0)
        .as_string()
        .ok_or_else(|| JsNativeError::typ().with_message("First argument must be a string"))?
        .to_std_string_escaped();

    let (use_url, last_chunk) = parse_base64_options(args.get_or_undefined(1), context)?;

    let target_length = get_uint8_length(&ta)?;

    let result = spec_from_base64(&input, use_url, last_chunk, Some(target_length));

    let written = result.bytes.len();
    if written > 0 {
        write_uint8_bytes(&ta, &result.bytes)?;
    }

    if let Some(err) = result.error {
        return Err(JsNativeError::syntax().with_message(err).into());
    }

    let obj = JsObject::with_null_proto();
    obj.create_data_property_or_throw(js_string!("read"), result.read, context)?;
    obj.create_data_property_or_throw(js_string!("written"), written, context)?;
    Ok(obj.into())
}

/// `Uint8Array.prototype.setFromHex ( string )`
pub(super) fn set_from_hex(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ta = validate_uint8array(this)?;

    let input = args
        .get_or_undefined(0)
        .as_string()
        .ok_or_else(|| JsNativeError::typ().with_message("First argument must be a string"))?
        .to_std_string_escaped();

    let target_length = get_uint8_length(&ta)?;

    let result = spec_from_hex(&input, Some(target_length));

    let written = result.bytes.len();
    if written > 0 {
        write_uint8_bytes(&ta, &result.bytes)?;
    }

    if let Some(err) = result.error {
        return Err(JsNativeError::syntax().with_message(err).into());
    }

    let obj = JsObject::with_null_proto();
    obj.create_data_property_or_throw(js_string!("read"), result.read, context)?;
    obj.create_data_property_or_throw(js_string!("written"), written, context)?;
    Ok(obj.into())
}
