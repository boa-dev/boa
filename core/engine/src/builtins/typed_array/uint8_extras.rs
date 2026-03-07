//! Implementation of the Uint8Array Base64/Hex proposal (TC39 Stage 4).
//!
//! This module adds the following methods:
//! - `Uint8Array.prototype.toBase64([ options ])`
//! - `Uint8Array.prototype.toHex()`
//! - `Uint8Array.prototype.setFromBase64(string [, options])`
//! - `Uint8Array.prototype.setFromHex(string)`
//! - `Uint8Array.fromBase64(string [, options])`
//! - `Uint8Array.fromHex(string)`
//!
//! Spec: <https://tc39.es/proposal-arraybuffer-base64/spec/>

use std::sync::atomic::Ordering;

use crate::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue,
    builtins::{BuiltInConstructor, typed_array::TypedArray},
    js_string,
    property::Attribute,
    value::JsVariant,
};

use crate::builtins::array_buffer::utils::SliceRef;

// ===== Constants =====

const BASE64_STANDARD: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const BASE64_URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

// ===== Helper: ValidateUint8Array =====

fn validate_uint8_array(this: &JsValue) -> JsResult<JsObject> {
    let obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Value is not a Uint8Array"))?;

    let ta = obj
        .downcast_ref::<TypedArray>()
        .ok_or_else(|| JsNativeError::typ().with_message("Value is not a Uint8Array"))?;

    if ta.kind().js_name() != js_string!("Uint8Array") {
        return Err(JsNativeError::typ()
            .with_message("Value is not a Uint8Array")
            .into());
    }

    drop(ta);
    Ok(obj.clone())
}

// ===== Helper: GetUint8ArrayBytes =====

fn read_bytes_from_slice_ref(data: &SliceRef<'_>, offset: usize, len: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(len);
    match data {
        SliceRef::Slice(s) => {
            bytes.extend_from_slice(&s[offset..offset + len]);
        }
        SliceRef::AtomicSlice(s) => {
            for i in offset..offset + len {
                bytes.push(s[i].load(Ordering::SeqCst));
            }
        }
    }
    bytes
}

fn get_uint8_array_bytes(obj: &JsObject) -> JsResult<Vec<u8>> {
    let ta = obj
        .downcast_ref::<TypedArray>()
        .ok_or_else(|| JsNativeError::typ().with_message("Value is not a Uint8Array"))?;

    let buf = ta.viewed_array_buffer().as_buffer();
    let Some(data) = buf
        .bytes(Ordering::SeqCst)
        .filter(|s| !ta.is_out_of_bounds(s.len()))
    else {
        return Err(JsNativeError::typ()
            .with_message("typed array is outside the bounds of its inner buffer")
            .into());
    };

    let byte_offset = ta.byte_offset() as usize;
    let len = ta.array_length(data.len()) as usize;

    Ok(read_bytes_from_slice_ref(&data, byte_offset, len))
}

// ===== Helper: SetUint8ArrayBytes =====

fn set_uint8_array_bytes(obj: &JsObject, bytes: &[u8]) -> JsResult<()> {
    let ta = obj
        .downcast_ref::<TypedArray>()
        .ok_or_else(|| JsNativeError::typ().with_message("Value is not a Uint8Array"))?;

    let buf_obj = ta.viewed_array_buffer().clone();
    let byte_offset = ta.byte_offset() as usize;
    drop(ta);

    let mut buf = buf_obj.as_buffer_mut();
    let Some(mut data) = buf.bytes(Ordering::SeqCst) else {
        return Err(JsNativeError::typ()
            .with_message("ArrayBuffer is detached")
            .into());
    };

    match &mut data {
        crate::builtins::array_buffer::utils::SliceRefMut::Slice(s) => {
            for (i, &byte) in bytes.iter().enumerate() {
                s[byte_offset + i] = byte;
            }
        }
        crate::builtins::array_buffer::utils::SliceRefMut::AtomicSlice(s) => {
            for (i, &byte) in bytes.iter().enumerate() {
                s[byte_offset + i].store(byte, Ordering::SeqCst);
            }
        }
    }

    Ok(())
}

// ===== Helper: GetOptionsObject =====

fn get_options_object(options: &JsValue, _context: &mut Context) -> JsResult<JsObject> {
    if options.is_undefined() {
        return Ok(JsObject::with_null_proto());
    }

    match options.variant() {
        JsVariant::Object(obj) => Ok(obj),
        _ => Err(JsNativeError::typ()
            .with_message("options must be an object")
            .into()),
    }
}

// ===== Helper: SkipAsciiWhitespace =====

fn skip_ascii_whitespace(string: &[u16], index: usize) -> usize {
    let length = string.len();
    let mut i = index;
    while i < length {
        let ch = string[i];
        if ch != 0x0009 && ch != 0x000A && ch != 0x000C && ch != 0x000D && ch != 0x0020 {
            return i;
        }
        i += 1;
    }
    i
}

// ===== Helper: base64 char value =====

fn base64_char_to_value(ch: u16) -> Option<u8> {
    match ch {
        0x41..=0x5A => Some((ch - 0x41) as u8),      // A-Z => 0-25
        0x61..=0x7A => Some((ch - 0x61 + 26) as u8), // a-z => 26-51
        0x30..=0x39 => Some((ch - 0x30 + 52) as u8), // 0-9 => 52-61
        0x2B => Some(62),                            // +
        0x2F => Some(63),                            // /
        _ => None,
    }
}

// ===== Helper: DecodeBase64Chunk =====

fn decode_base64_chunk(chunk: &[u16], throw_on_extra_bits: Option<bool>) -> JsResult<Vec<u8>> {
    let chunk_length = chunk.len();

    let padded: Vec<u16> = if chunk_length == 2 {
        vec![chunk[0], chunk[1], b'A' as u16, b'A' as u16]
    } else if chunk_length == 3 {
        vec![chunk[0], chunk[1], chunk[2], b'A' as u16]
    } else {
        debug_assert!(chunk_length == 4);
        chunk.to_vec()
    };

    let mut n: u32 = 0;
    for &ch in &padded {
        let val = base64_char_to_value(ch)
            .ok_or_else(|| JsNativeError::syntax().with_message("invalid base64 character"))?;
        n = (n << 6) | u32::from(val);
    }

    let b0 = ((n >> 16) & 0xFF) as u8;
    let b1 = ((n >> 8) & 0xFF) as u8;
    let b2 = (n & 0xFF) as u8;

    if chunk_length == 2 {
        if throw_on_extra_bits.unwrap_or(false) && b1 != 0 {
            return Err(JsNativeError::syntax()
                .with_message("extra bits in base64 chunk")
                .into());
        }
        Ok(vec![b0])
    } else if chunk_length == 3 {
        if throw_on_extra_bits.unwrap_or(false) && b2 != 0 {
            return Err(JsNativeError::syntax()
                .with_message("extra bits in base64 chunk")
                .into());
        }
        Ok(vec![b0, b1])
    } else {
        Ok(vec![b0, b1, b2])
    }
}

// ===== FromBase64 =====

struct FromBase64Result {
    read: usize,
    bytes: Vec<u8>,
    error: Option<crate::JsError>,
}

fn from_base64(
    string: &[u16],
    alphabet: &str,
    last_chunk_handling: &str,
    max_length: Option<usize>,
) -> FromBase64Result {
    let max_length = max_length.unwrap_or(usize::MAX);

    if max_length == 0 {
        return FromBase64Result {
            read: 0,
            bytes: Vec::new(),
            error: None,
        };
    }

    let mut read: usize = 0;
    let mut bytes: Vec<u8> = Vec::new();
    let mut chunk: Vec<u16> = Vec::new();
    let mut index: usize = 0;
    let length = string.len();

    loop {
        index = skip_ascii_whitespace(string, index);

        if index == length {
            if !chunk.is_empty() {
                if last_chunk_handling == "stop-before-partial" {
                    return FromBase64Result {
                        read,
                        bytes,
                        error: None,
                    };
                } else if last_chunk_handling == "loose" {
                    if chunk.len() == 1 {
                        return FromBase64Result {
                            read,
                            bytes,
                            error: Some(
                                JsNativeError::syntax()
                                    .with_message("invalid base64 string: incomplete chunk")
                                    .into(),
                            ),
                        };
                    }
                    match decode_base64_chunk(&chunk, Some(false)) {
                        Ok(result) => bytes.extend(result),
                        Err(e) => {
                            return FromBase64Result {
                                read,
                                bytes,
                                error: Some(e),
                            };
                        }
                    }
                } else {
                    // strict
                    return FromBase64Result {
                        read,
                        bytes,
                        error: Some(
                            JsNativeError::syntax()
                                .with_message(
                                    "invalid base64 string: incomplete chunk in strict mode",
                                )
                                .into(),
                        ),
                    };
                }
            }
            return FromBase64Result {
                read: length,
                bytes,
                error: None,
            };
        }

        let char_code = string[index];
        index += 1;

        // Handle padding '='
        if char_code == b'=' as u16 {
            if chunk.len() < 2 {
                return FromBase64Result {
                    read,
                    bytes,
                    error: Some(
                        JsNativeError::syntax()
                            .with_message("invalid base64 string: padding in unexpected position")
                            .into(),
                    ),
                };
            }

            index = skip_ascii_whitespace(string, index);

            if chunk.len() == 2 {
                if index == length {
                    if last_chunk_handling == "stop-before-partial" {
                        return FromBase64Result {
                            read,
                            bytes,
                            error: None,
                        };
                    }
                    return FromBase64Result {
                        read,
                        bytes,
                        error: Some(
                            JsNativeError::syntax()
                                .with_message(
                                    "invalid base64 string: missing second padding character",
                                )
                                .into(),
                        ),
                    };
                }

                let next_char = string[index];
                if next_char == b'=' as u16 {
                    index = skip_ascii_whitespace(string, index + 1);
                }
            }

            if index < length {
                return FromBase64Result {
                    read,
                    bytes,
                    error: Some(
                        JsNativeError::syntax()
                            .with_message("invalid base64 string: characters after padding")
                            .into(),
                    ),
                };
            }

            let throw_on = last_chunk_handling == "strict";
            match decode_base64_chunk(&chunk, Some(throw_on)) {
                Ok(result) => {
                    bytes.extend(result);
                    return FromBase64Result {
                        read: length,
                        bytes,
                        error: None,
                    };
                }
                Err(e) => {
                    return FromBase64Result {
                        read,
                        bytes,
                        error: Some(e),
                    };
                }
            }
        }

        // Handle base64url alphabet mapping
        let mut mapped_char = char_code;
        if alphabet == "base64url" {
            if char_code == b'+' as u16 || char_code == b'/' as u16 {
                return FromBase64Result {
                    read,
                    bytes,
                    error: Some(
                        JsNativeError::syntax()
                            .with_message("invalid base64url character: + or / not allowed")
                            .into(),
                    ),
                };
            }
            if char_code == b'-' as u16 {
                mapped_char = b'+' as u16;
            } else if char_code == b'_' as u16 {
                mapped_char = b'/' as u16;
            }
        }

        // Check if it's a valid base64 character
        if base64_char_to_value(mapped_char).is_none() {
            return FromBase64Result {
                read,
                bytes,
                error: Some(
                    JsNativeError::syntax()
                        .with_message("invalid base64 character")
                        .into(),
                ),
            };
        }

        // Check remaining capacity
        let remaining = max_length - bytes.len();
        if (remaining == 1 && chunk.len() == 2) || (remaining == 2 && chunk.len() == 3) {
            return FromBase64Result {
                read,
                bytes,
                error: None,
            };
        }

        chunk.push(mapped_char);

        if chunk.len() == 4 {
            match decode_base64_chunk(&chunk, None) {
                Ok(result) => bytes.extend(result),
                Err(e) => {
                    return FromBase64Result {
                        read,
                        bytes,
                        error: Some(e),
                    };
                }
            }
            chunk.clear();
            read = index;

            if bytes.len() == max_length {
                return FromBase64Result {
                    read,
                    bytes,
                    error: None,
                };
            }
        }
    }
}

// ===== FromHex =====

struct FromHexResult {
    read: usize,
    bytes: Vec<u8>,
    error: Option<crate::JsError>,
}

fn hex_char_to_value(ch: u16) -> Option<u8> {
    match ch {
        0x30..=0x39 => Some((ch - 0x30) as u8),      // '0'-'9'
        0x41..=0x46 => Some((ch - 0x41 + 10) as u8), // 'A'-'F'
        0x61..=0x66 => Some((ch - 0x61 + 10) as u8), // 'a'-'f'
        _ => None,
    }
}

fn from_hex(string: &[u16], max_length: Option<usize>) -> FromHexResult {
    let max_length = max_length.unwrap_or(usize::MAX);
    let length = string.len();
    let mut bytes: Vec<u8> = Vec::new();
    let mut read: usize = 0;

    if length % 2 != 0 {
        return FromHexResult {
            read,
            bytes,
            error: Some(
                JsNativeError::syntax()
                    .with_message("hex string must have an even number of characters")
                    .into(),
            ),
        };
    }

    while read < length && bytes.len() < max_length {
        let h1 = string[read];
        let h2 = string[read + 1];

        let v1 = hex_char_to_value(h1);
        let v2 = hex_char_to_value(h2);

        if v1.is_none() || v2.is_none() {
            return FromHexResult {
                read,
                bytes,
                error: Some(
                    JsNativeError::syntax()
                        .with_message("invalid hex character")
                        .into(),
                ),
            };
        }

        read += 2;
        let byte = (v1.unwrap() << 4) | v2.unwrap();
        bytes.push(byte);
    }

    FromHexResult {
        read,
        bytes,
        error: None,
    }
}

// ===== Public methods =====

/// `Uint8Array.prototype.toBase64 ( [ options ] )`
pub(crate) fn to_base64(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = validate_uint8_array(this)?;

    let opts = get_options_object(args.get_or_undefined(0), context)?;

    let alphabet_val = opts.get(js_string!("alphabet"), context)?;
    let alphabet = if alphabet_val.is_undefined() {
        "base64".to_string()
    } else {
        alphabet_val
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("alphabet must be a string"))?
            .to_std_string_escaped()
    };

    if alphabet != "base64" && alphabet != "base64url" {
        return Err(JsNativeError::typ()
            .with_message("alphabet must be either \"base64\" or \"base64url\"")
            .into());
    }

    let omit_padding_val = opts.get(js_string!("omitPadding"), context)?;
    let omit_padding = omit_padding_val.to_boolean();

    let to_encode = get_uint8_array_bytes(&obj)?;

    let table = if alphabet == "base64" {
        BASE64_STANDARD
    } else {
        BASE64_URL
    };

    let mut out = String::new();

    for chunk in to_encode.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let n = (b0 << 16) | (b1 << 8) | b2;

        out.push(table[((n >> 18) & 0x3F) as usize] as char);
        out.push(table[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            out.push(table[((n >> 6) & 0x3F) as usize] as char);
        } else if !omit_padding {
            out.push('=');
        }

        if chunk.len() > 2 {
            out.push(table[(n & 0x3F) as usize] as char);
        } else if !omit_padding {
            out.push('=');
        }
    }

    Ok(JsValue::new(js_string!(out.as_str())))
}

/// `Uint8Array.prototype.toHex ( )`
pub(crate) fn to_hex(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = validate_uint8_array(this)?;
    let to_encode = get_uint8_array_bytes(&obj)?;

    let mut out = String::with_capacity(to_encode.len() * 2);
    for &byte in &to_encode {
        out.push_str(&format!("{byte:02x}"));
    }

    Ok(JsValue::new(js_string!(out.as_str())))
}

/// `Uint8Array.fromBase64 ( string [, options] )`
pub(crate) fn from_base64_static(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let js_str = args
        .get_or_undefined(0)
        .as_string()
        .ok_or_else(|| JsNativeError::typ().with_message("first argument must be a string"))?;

    let opts = get_options_object(args.get_or_undefined(1), context)?;

    let alphabet_val = opts.get(js_string!("alphabet"), context)?;
    let alphabet = if alphabet_val.is_undefined() {
        "base64".to_string()
    } else {
        alphabet_val
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("alphabet must be a string"))?
            .to_std_string_escaped()
    };
    if alphabet != "base64" && alphabet != "base64url" {
        return Err(JsNativeError::typ()
            .with_message("alphabet must be either \"base64\" or \"base64url\"")
            .into());
    }

    let lch_val = opts.get(js_string!("lastChunkHandling"), context)?;
    let last_chunk_handling = if lch_val.is_undefined() {
        "loose".to_string()
    } else {
        lch_val
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("lastChunkHandling must be a string"))?
            .to_std_string_escaped()
    };
    if !["loose", "strict", "stop-before-partial"].contains(&last_chunk_handling.as_str()) {
        return Err(JsNativeError::typ()
            .with_message("lastChunkHandling must be one of \"loose\", \"strict\", or \"stop-before-partial\"")
            .into());
    }

    let str_vec: Vec<u16> = js_str.iter().collect();
    let result = from_base64(&str_vec, &alphabet, &last_chunk_handling, None);

    if let Some(error) = result.error {
        return Err(error);
    }

    let result_length = result.bytes.len();
    let constructor = context
        .intrinsics()
        .constructors()
        .typed_uint8_array()
        .constructor();

    let ta = crate::builtins::typed_array::Uint8Array::constructor(
        &constructor.into(),
        &[JsValue::new(result_length as u64)],
        context,
    )?;

    let ta_obj = ta
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("failed to create Uint8Array"))?;

    set_uint8_array_bytes(&ta_obj, &result.bytes)?;

    Ok(ta)
}

/// `Uint8Array.fromHex ( string )`
pub(crate) fn from_hex_static(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let js_str = args
        .get_or_undefined(0)
        .as_string()
        .ok_or_else(|| JsNativeError::typ().with_message("first argument must be a string"))?;

    let str_vec: Vec<u16> = js_str.iter().collect();
    let result = from_hex(&str_vec, None);

    if let Some(error) = result.error {
        return Err(error);
    }

    let result_length = result.bytes.len();
    let constructor = context
        .intrinsics()
        .constructors()
        .typed_uint8_array()
        .constructor();

    let ta = crate::builtins::typed_array::Uint8Array::constructor(
        &constructor.into(),
        &[JsValue::new(result_length as u64)],
        context,
    )?;

    let ta_obj = ta
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("failed to create Uint8Array"))?;

    set_uint8_array_bytes(&ta_obj, &result.bytes)?;

    Ok(ta)
}

/// `Uint8Array.prototype.setFromBase64 ( string [, options] )`
pub(crate) fn set_from_base64(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let into = validate_uint8_array(this)?;

    let js_str = args
        .get_or_undefined(0)
        .as_string()
        .ok_or_else(|| JsNativeError::typ().with_message("first argument must be a string"))?;

    let opts = get_options_object(args.get_or_undefined(1), context)?;

    let alphabet_val = opts.get(js_string!("alphabet"), context)?;
    let alphabet = if alphabet_val.is_undefined() {
        "base64".to_string()
    } else {
        alphabet_val
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("alphabet must be a string"))?
            .to_std_string_escaped()
    };
    if alphabet != "base64" && alphabet != "base64url" {
        return Err(JsNativeError::typ()
            .with_message("alphabet must be either \"base64\" or \"base64url\"")
            .into());
    }

    let lch_val = opts.get(js_string!("lastChunkHandling"), context)?;
    let last_chunk_handling = if lch_val.is_undefined() {
        "loose".to_string()
    } else {
        lch_val
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("lastChunkHandling must be a string"))?
            .to_std_string_escaped()
    };
    if !["loose", "strict", "stop-before-partial"].contains(&last_chunk_handling.as_str()) {
        return Err(JsNativeError::typ()
            .with_message("lastChunkHandling must be one of \"loose\", \"strict\", or \"stop-before-partial\"")
            .into());
    }

    let byte_length = {
        let ta = into
            .downcast_ref::<TypedArray>()
            .ok_or_else(|| JsNativeError::typ().with_message("Value is not a Uint8Array"))?;
        let buf = ta.viewed_array_buffer().as_buffer();
        let Some(data) = buf
            .bytes(Ordering::SeqCst)
            .filter(|s| !ta.is_out_of_bounds(s.len()))
        else {
            return Err(JsNativeError::typ()
                .with_message("typed array is outside the bounds of its inner buffer")
                .into());
        };
        ta.array_length(data.len()) as usize
    };

    let str_vec: Vec<u16> = js_str.iter().collect();
    let result = from_base64(&str_vec, &alphabet, &last_chunk_handling, Some(byte_length));

    let written = result.bytes.len();
    set_uint8_array_bytes(&into, &result.bytes)?;

    if let Some(error) = result.error {
        return Err(error);
    }

    let result_object = crate::object::ObjectInitializer::new(context)
        .property(
            js_string!("read"),
            JsValue::new(result.read as u64),
            Attribute::all(),
        )
        .property(
            js_string!("written"),
            JsValue::new(written as u64),
            Attribute::all(),
        )
        .build();

    Ok(JsValue::from(result_object))
}

/// `Uint8Array.prototype.setFromHex ( string )`
pub(crate) fn set_from_hex(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let into = validate_uint8_array(this)?;

    let js_str = args
        .get_or_undefined(0)
        .as_string()
        .ok_or_else(|| JsNativeError::typ().with_message("first argument must be a string"))?;

    let byte_length = {
        let ta = into
            .downcast_ref::<TypedArray>()
            .ok_or_else(|| JsNativeError::typ().with_message("Value is not a Uint8Array"))?;
        let buf = ta.viewed_array_buffer().as_buffer();
        let Some(data) = buf
            .bytes(Ordering::SeqCst)
            .filter(|s| !ta.is_out_of_bounds(s.len()))
        else {
            return Err(JsNativeError::typ()
                .with_message("typed array is outside the bounds of its inner buffer")
                .into());
        };
        ta.array_length(data.len()) as usize
    };

    let str_vec: Vec<u16> = js_str.iter().collect();
    let result = from_hex(&str_vec, Some(byte_length));

    let written = result.bytes.len();
    set_uint8_array_bytes(&into, &result.bytes)?;

    if let Some(error) = result.error {
        return Err(error);
    }

    let result_object = crate::object::ObjectInitializer::new(context)
        .property(
            js_string!("read"),
            JsValue::new(result.read as u64),
            Attribute::all(),
        )
        .property(
            js_string!("written"),
            JsValue::new(written as u64),
            Attribute::all(),
        )
        .build();

    Ok(JsValue::from(result_object))
}
