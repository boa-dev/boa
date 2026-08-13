//! `Uint8Array`-specific base64 and hex encoding/decoding methods.
//!
//! Implements the [proposal-arraybuffer-base64](https://tc39.es/proposal-arraybuffer-base64/) methods:
//! - [`Uint8Array.fromBase64()`](https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.frombase64)
//! - [`Uint8Array.prototype.setFromBase64()`](https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.prototype.setfrombase64)
//! - [`Uint8Array.prototype.toBase64()`](https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.prototype.tobase64)
//! - [`Uint8Array.fromHex()`](https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.fromhex)
//! - [`Uint8Array.prototype.setFromHex()`](https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.prototype.setfromhex)
//! - [`Uint8Array.prototype.toHex()`](https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.prototype.tohex)

use std::{cmp::min, sync::atomic::Ordering};

use super::{
    TypedArray, TypedArrayKind,
    base64::{self, Alphabet as Base64Alphabet, LastChunkHandling as Base64LastChunkHandling},
    hex,
};
use crate::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsValue,
    builtins::array_buffer::{ArrayBuffer, BufferObject, utils::SliceRefMut},
    js_string,
};

/// Boa's implementation of `Uint8Array`-specific base64 and hex proposal methods.
pub(crate) struct BuiltinUint8Array;

impl BuiltinUint8Array {
    /// `Uint8Array.fromBase64 ( string, options )`
    ///
    /// More information:
    ///  - [proposal-arraybuffer-base64][spec]
    ///
    /// [spec]: https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.frombase64
    pub(crate) fn from_base64(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If string is not a String, throw a TypeError exception.
        let input = args.get_or_undefined(0);
        // Check if input is a string (not a string object) - match V8's behavior
        let Some(input_string) = input.as_string() else {
            return Err(JsNativeError::typ()
                .with_message("input must be a string")
                .into());
        };

        // 2. Let opts be ? GetOptionsObject(options).
        let options = args.get_or_undefined(1);
        let (alphabet, last_chunk_handling) = Self::get_base64_options(options, context)?;

        let input_bytes = Self::encoded_input_bytes(&input_string);
        let decoded = base64::decode(&input_bytes, alphabet, last_chunk_handling, None);

        if decoded.error.is_some() {
            return Err(JsNativeError::syntax()
                .with_message("Invalid base64 string")
                .into());
        }

        let output_len = decoded.output.len();

        // Create Uint8Array from decoded data
        let buffer = ArrayBuffer::allocate(
            &context
                .intrinsics()
                .constructors()
                .array_buffer()
                .constructor()
                .into(),
            output_len as u64,
            None,
            context,
        )?;

        // Copy data to buffer
        {
            let mut buffer_data = buffer.borrow_mut();
            if let Some(bytes) = buffer_data.data_mut().bytes_mut() {
                bytes[..output_len].copy_from_slice(&decoded.output);
            }
        }

        // Create Uint8Array
        let uint8_array = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .constructors()
                .typed_uint8_array()
                .prototype(),
            TypedArray::new(
                BufferObject::Buffer(buffer),
                TypedArrayKind::Uint8,
                0,
                Some(output_len as u64),
                Some(output_len as u64),
            ),
        );

        Ok(uint8_array.upcast().into())
    }

    /// `Uint8Array.prototype.setFromBase64 ( string, options )`
    ///
    /// More information:
    ///  - [proposal-arraybuffer-base64][spec]
    ///
    /// [spec]: https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.prototype.setfrombase64
    pub(crate) fn set_from_base64(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let into be the this value.
        // 2. Perform ? ValidateUint8Array(into).
        let uint8array = this
            .as_object()
            .and_then(|o| o.clone().downcast::<TypedArray>().ok())
            .ok_or_else(|| JsNativeError::typ().with_message("Value is not a Uint8Array object"))?;

        let uint8array_borrow = uint8array.borrow();
        let uint8array_data = uint8array_borrow.data();

        // Verify it's a Uint8Array
        if uint8array_data.kind() != TypedArrayKind::Uint8 {
            return Err(JsNativeError::typ()
                .with_message("Value is not a Uint8Array object")
                .into());
        }

        // Check if detached
        let buffer = uint8array_data.viewed_array_buffer();
        let Some(buf_len) = buffer.as_buffer().bytes(Ordering::SeqCst).map(|s| s.len()) else {
            return Err(JsNativeError::typ()
                .with_message("TypedArray is detached")
                .into());
        };

        if uint8array_data.is_out_of_bounds(buf_len) {
            return Err(JsNativeError::typ()
                .with_message("TypedArray is out of bounds")
                .into());
        }

        let array_length = uint8array_data.array_length(buf_len);
        let byte_offset = uint8array_data.byte_offset() as usize;
        drop(uint8array_borrow);

        // 3. If string is not a String, throw a TypeError exception.
        let input = args.get_or_undefined(0);
        // Check if input is a string (not a string object) - match V8's behavior
        let Some(input_string) = input.as_string() else {
            return Err(JsNativeError::typ()
                .with_message("input must be a string")
                .into());
        };

        // 4. Let opts be ? GetOptionsObject(options).
        let options = args.get_or_undefined(1);
        let (alphabet, last_chunk_handling) = Self::get_base64_options(options, context)?;

        // If array length is 0, return early
        if array_length == 0 {
            let read = JsValue::from(0);
            let written = JsValue::from(0);
            return Self::create_set_from_result(read, written, context);
        }

        let input_bytes = Self::encoded_input_bytes(&input_string);
        let mut output = vec![0; 6 * input_bytes.len() / 8];
        let result = base64::decode_mut(
            &input_bytes,
            &mut output,
            alphabet,
            last_chunk_handling,
            Some(array_length as usize),
        );

        // FromBase64 does not invoke user code, so the backing buffer cannot be detached or shrunk
        // between the bounds check above and the copy below.
        {
            let uint8array_mut = uint8array.borrow_mut();
            let uint8array_data = uint8array_mut.data();
            let mut buffer = uint8array_data.viewed_array_buffer().as_buffer_mut();
            let Some(mut data) = buffer.bytes(Ordering::SeqCst) else {
                return Err(JsNativeError::typ()
                    .with_message("Cannot access buffer data")
                    .into());
            };

            let mut subslice = data.subslice_mut(byte_offset..byte_offset + array_length as usize);
            match &mut subslice {
                SliceRefMut::Slice(slice) => {
                    slice[..result.written].copy_from_slice(&output[..result.written]);
                }
                SliceRefMut::AtomicSlice(slice) => {
                    for (dst, src) in slice.iter().zip(&output[..result.written]) {
                        dst.store(*src, Ordering::SeqCst);
                    }
                }
            }
        }

        if result.error.is_some() {
            return Err(JsNativeError::syntax()
                .with_message("Invalid base64 string")
                .into());
        }

        let read = JsValue::from(result.read as u64);
        let written = JsValue::from(result.written as u64);
        Self::create_set_from_result(read, written, context)
    }

    /// `Uint8Array.prototype.toBase64 ( options )`
    ///
    /// More information:
    ///  - [proposal-arraybuffer-base64][spec]
    ///
    /// [spec]: https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.prototype.tobase64
    pub(crate) fn to_base64(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateUint8Array(O).
        let uint8array = this
            .as_object()
            .and_then(|o| o.clone().downcast::<TypedArray>().ok())
            .ok_or_else(|| JsNativeError::typ().with_message("Value is not a Uint8Array object"))?;

        let uint8array_borrow = uint8array.borrow();
        let uint8array_data = uint8array_borrow.data();

        // Verify it's a Uint8Array
        if uint8array_data.kind() != TypedArrayKind::Uint8 {
            return Err(JsNativeError::typ()
                .with_message("Value is not a Uint8Array object")
                .into());
        }

        // Get buffer info but don't check detached yet
        let byte_offset = uint8array_data.byte_offset() as usize;
        drop(uint8array_borrow);

        // 3. Let opts be ? GetOptionsObject(options).
        // Get options first (this may trigger side effects that detach the buffer)
        let options = args.get_or_undefined(0);
        let (alphabet, omit_padding) = Self::get_base64_encode_options(options, context)?;

        // After getting options, check if buffer is detached
        let uint8array_borrow = uint8array.borrow();
        let uint8array_data = uint8array_borrow.data();
        let buffer = uint8array_data.viewed_array_buffer();
        let Some(buf_len) = buffer.as_buffer().bytes(Ordering::SeqCst).map(|s| s.len()) else {
            return Err(JsNativeError::typ()
                .with_message("TypedArray is detached")
                .into());
        };

        if uint8array_data.is_out_of_bounds(buf_len) {
            return Err(JsNativeError::typ()
                .with_message("TypedArray is out of bounds")
                .into());
        }

        let byte_length = uint8array_data.array_length(buf_len) as usize;

        // Get the data
        let buffer_data = buffer.as_buffer();
        let Some(data) = buffer_data.bytes(Ordering::SeqCst) else {
            return Err(JsNativeError::typ()
                .with_message("Cannot access buffer data")
                .into());
        };

        let input = data
            .subslice(byte_offset..byte_offset + byte_length)
            .to_vec();
        Ok(JsString::from(base64::encode(&input, alphabet, omit_padding)).into())
    }

    /// `Uint8Array.fromHex ( string )`
    ///
    /// More information:
    ///  - [proposal-arraybuffer-base64][spec]
    ///
    /// [spec]: https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.fromhex
    pub(crate) fn from_hex(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If string is not a String, throw a TypeError exception.
        let input = args.get_or_undefined(0);
        // Check if input is a string (not a string object) - match V8's behavior
        let Some(input_string) = input.as_string() else {
            return Err(JsNativeError::typ()
                .with_message("input must be a string")
                .into());
        };

        let input_bytes = Self::encoded_input_bytes(&input_string);

        // Check if length is even
        if !input_bytes.len().is_multiple_of(2) {
            return Err(JsNativeError::syntax()
                .with_message("Invalid hex string: odd length")
                .into());
        }

        let decoded = hex::decode(&input_bytes, None);
        if decoded.error.is_some() {
            return Err(JsNativeError::syntax()
                .with_message("Invalid hex character")
                .into());
        }
        let output_len = decoded.output.len();

        // Create Uint8Array from decoded data
        let buffer = ArrayBuffer::allocate(
            &context
                .intrinsics()
                .constructors()
                .array_buffer()
                .constructor()
                .into(),
            output_len as u64,
            None,
            context,
        )?;

        // Copy data to buffer
        {
            let mut buffer_data = buffer.borrow_mut();
            if let Some(bytes) = buffer_data.data_mut().bytes_mut() {
                bytes[..output_len].copy_from_slice(&decoded.output);
            }
        }

        // Create Uint8Array
        let uint8_array = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .constructors()
                .typed_uint8_array()
                .prototype(),
            TypedArray::new(
                BufferObject::Buffer(buffer),
                TypedArrayKind::Uint8,
                0,
                Some(output_len as u64),
                Some(output_len as u64),
            ),
        );

        Ok(uint8_array.upcast().into())
    }

    /// `Uint8Array.prototype.setFromHex ( string )`
    ///
    /// More information:
    ///  - [proposal-arraybuffer-base64][spec]
    ///
    /// [spec]: https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.prototype.setfromhex
    pub(crate) fn set_from_hex(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let into be the this value.
        // 2. Perform ? ValidateUint8Array(into).
        let uint8array = this
            .as_object()
            .and_then(|o| o.clone().downcast::<TypedArray>().ok())
            .ok_or_else(|| JsNativeError::typ().with_message("Value is not a Uint8Array object"))?;

        let uint8array_borrow = uint8array.borrow();
        let uint8array_data = uint8array_borrow.data();

        // Verify it's a Uint8Array
        if uint8array_data.kind() != TypedArrayKind::Uint8 {
            return Err(JsNativeError::typ()
                .with_message("Value is not a Uint8Array object")
                .into());
        }

        // Check if detached
        let buffer = uint8array_data.viewed_array_buffer();
        let Some(buf_len) = buffer.as_buffer().bytes(Ordering::SeqCst).map(|s| s.len()) else {
            return Err(JsNativeError::typ()
                .with_message("TypedArray is detached")
                .into());
        };

        if uint8array_data.is_out_of_bounds(buf_len) {
            return Err(JsNativeError::typ()
                .with_message("TypedArray is out of bounds")
                .into());
        }

        let array_length = uint8array_data.array_length(buf_len);
        let byte_offset = uint8array_data.byte_offset() as usize;
        drop(uint8array_borrow);

        // 3. If string is not a String, throw a TypeError exception.
        let input = args.get_or_undefined(0);
        // Check if input is a string (not a string object) - match V8's behavior
        let Some(input_string) = input.as_string() else {
            return Err(JsNativeError::typ()
                .with_message("input must be a string")
                .into());
        };

        let input_bytes = Self::encoded_input_bytes(&input_string);
        let input_len = input_bytes.len();

        // Check if length is odd first - this must be done even if array_length is 0
        // Per spec: FromHex checks length before checking maxLength
        if !input_len.is_multiple_of(2) {
            return Err(JsNativeError::syntax()
                .with_message("Invalid hex string: odd length")
                .into());
        }

        // If array length is 0, return early (after checking for odd length)
        if array_length == 0 {
            let read = JsValue::from(0);
            let written = JsValue::from(0);
            return Self::create_set_from_result(read, written, context);
        }

        // 4. Let taRecord be MakeTypedArrayWithBufferWitnessRecord(into, seq-cst).
        // 5. If IsTypedArrayOutOfBounds(taRecord) is true, throw a TypeError exception.
        // 6. Let byteLength be TypedArrayLength(taRecord).
        // 7. Let result be FromHex(string, byteLength).
        // 8. Let bytes be result.[[Bytes]].
        // 9. Let written be the length of bytes.
        // 10. NOTE: FromHex does not invoke any user code, so the ArrayBuffer backing
        //     into cannot have been detached or shrunk.
        // 11. Assert: written ≤ byteLength.
        // 12. Perform SetUint8ArrayBytes(into, bytes).
        // 13. If result.[[Error]] is not none, then
        //     a. Throw result.[[Error]].
        // 14. Let resultObject be OrdinaryObjectCreate(%Object.prototype%).
        // 15. Perform ! CreateDataPropertyOrThrow(resultObject, "read", 𝔽(result.[[Read]])).
        // 16. Perform ! CreateDataPropertyOrThrow(resultObject, "written", 𝔽(written)).
        // 17. Return resultObject.

        let output_len = min(input_len / 2, array_length as usize);
        let mut output = vec![0; output_len];
        let result = hex::decode_mut(&input_bytes, &mut output, Some(array_length as usize));

        {
            let uint8array_mut = uint8array.borrow_mut();
            let uint8array_data = uint8array_mut.data();
            let mut buffer = uint8array_data.viewed_array_buffer().as_buffer_mut();
            let Some(mut data) = buffer.bytes(Ordering::SeqCst) else {
                return Err(JsNativeError::typ()
                    .with_message("Cannot access buffer data")
                    .into());
            };

            let mut subslice = data.subslice_mut(byte_offset..byte_offset + array_length as usize);
            match &mut subslice {
                SliceRefMut::Slice(slice) => {
                    slice[..result.written].copy_from_slice(&output[..result.written]);
                }
                SliceRefMut::AtomicSlice(slice) => {
                    for (dst, src) in slice.iter().zip(&output[..result.written]) {
                        dst.store(*src, Ordering::SeqCst);
                    }
                }
            }
        }

        if result.error.is_some() {
            return Err(JsNativeError::syntax()
                .with_message("Invalid hex character")
                .into());
        }

        let read = JsValue::from(result.read as u64);
        let written = JsValue::from(result.written as u64);
        Self::create_set_from_result(read, written, context)
    }

    /// `Uint8Array.prototype.toHex ( )`
    ///
    /// More information:
    ///  - [proposal-arraybuffer-base64][spec]
    ///
    /// [spec]: https://tc39.es/proposal-arraybuffer-base64/spec/#sec-uint8array.prototype.tohex
    pub(crate) fn to_hex(
        this: &JsValue,
        _: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateUint8Array(O).
        let uint8array = this
            .as_object()
            .and_then(|o| o.clone().downcast::<TypedArray>().ok())
            .ok_or_else(|| JsNativeError::typ().with_message("Value is not a Uint8Array object"))?;

        let uint8array_borrow = uint8array.borrow();
        let uint8array_data = uint8array_borrow.data();

        // Verify it's a Uint8Array
        if uint8array_data.kind() != TypedArrayKind::Uint8 {
            return Err(JsNativeError::typ()
                .with_message("Value is not a Uint8Array object")
                .into());
        }

        // Check if detached
        let buffer = uint8array_data.viewed_array_buffer();
        let Some(buf_len) = buffer.as_buffer().bytes(Ordering::SeqCst).map(|s| s.len()) else {
            return Err(JsNativeError::typ()
                .with_message("TypedArray is detached")
                .into());
        };

        if uint8array_data.is_out_of_bounds(buf_len) {
            return Err(JsNativeError::typ()
                .with_message("TypedArray is out of bounds")
                .into());
        }

        let byte_offset = uint8array_data.byte_offset() as usize;
        let byte_length = uint8array_data.array_length(buf_len) as usize;

        // Get the data
        let buffer_data = buffer.as_buffer();
        let Some(data) = buffer_data.bytes(Ordering::SeqCst) else {
            return Err(JsNativeError::typ()
                .with_message("Cannot access buffer data")
                .into());
        };

        let input = data
            .subslice(byte_offset..byte_offset + byte_length)
            .to_vec();
        Ok(JsString::from(hex::encode(&input)).into())
    }

    // ===== Private helpers =====

    fn encoded_input_bytes(input: &JsString) -> Vec<u8> {
        if let Some(bytes) = input.as_str().as_latin1() {
            return bytes.to_vec();
        }

        input
            .iter()
            .map(|code_unit| {
                if u8::try_from(code_unit).is_ok() {
                    code_unit as u8
                } else {
                    u8::MAX
                }
            })
            .collect()
    }

    fn get_base64_options(
        options: &JsValue,
        context: &mut Context,
    ) -> JsResult<(Base64Alphabet, Base64LastChunkHandling)> {
        let mut alphabet = Base64Alphabet::Base64;
        let mut last_chunk_handling = Base64LastChunkHandling::Loose;

        if let Some(options_obj) = options.as_object() {
            // Get alphabet option
            let alphabet_value = options_obj.get(js_string!("alphabet"), context)?;
            if !alphabet_value.is_undefined() {
                // Check if it's a string (not a string object)
                let Some(alphabet_str) = alphabet_value.as_string() else {
                    return Err(JsNativeError::typ()
                        .with_message("Invalid alphabet option")
                        .into());
                };
                if alphabet_str == js_string!("base64") {
                    alphabet = Base64Alphabet::Base64;
                } else if alphabet_str == js_string!("base64url") {
                    alphabet = Base64Alphabet::Base64Url;
                } else {
                    return Err(JsNativeError::typ()
                        .with_message("Invalid alphabet option")
                        .into());
                }
            }

            // Get lastChunkHandling option
            let last_chunk_value = options_obj.get(js_string!("lastChunkHandling"), context)?;
            if !last_chunk_value.is_undefined() {
                // Check if it's a string (not a string object) - match V8's behavior
                let Some(last_chunk_str) = last_chunk_value.as_string() else {
                    return Err(JsNativeError::typ()
                        .with_message("Invalid lastChunkHandling option")
                        .into());
                };
                if last_chunk_str == js_string!("loose") {
                    last_chunk_handling = Base64LastChunkHandling::Loose;
                } else if last_chunk_str == js_string!("strict") {
                    last_chunk_handling = Base64LastChunkHandling::Strict;
                } else if last_chunk_str == js_string!("stop-before-partial") {
                    last_chunk_handling = Base64LastChunkHandling::StopBeforePartial;
                } else {
                    return Err(JsNativeError::typ()
                        .with_message("Invalid lastChunkHandling option")
                        .into());
                }
            }
        }

        Ok((alphabet, last_chunk_handling))
    }

    fn get_base64_encode_options(
        options: &JsValue,
        context: &mut Context,
    ) -> JsResult<(Base64Alphabet, bool)> {
        let mut alphabet = Base64Alphabet::Base64;
        let mut omit_padding = false;

        if let Some(options_obj) = options.as_object() {
            // Get alphabet option
            let alphabet_value = options_obj.get(js_string!("alphabet"), context)?;
            if !alphabet_value.is_undefined() {
                // Check if it's a string (not a string object)
                let Some(alphabet_str) = alphabet_value.as_string() else {
                    return Err(JsNativeError::typ()
                        .with_message("Invalid alphabet option")
                        .into());
                };
                if alphabet_str == js_string!("base64") {
                    alphabet = Base64Alphabet::Base64;
                } else if alphabet_str == js_string!("base64url") {
                    alphabet = Base64Alphabet::Base64Url;
                } else {
                    return Err(JsNativeError::typ()
                        .with_message("Invalid alphabet option")
                        .into());
                }
            }

            // Get omitPadding option
            let omit_padding_value = options_obj.get(js_string!("omitPadding"), context)?;
            if !omit_padding_value.is_undefined() {
                omit_padding = omit_padding_value.to_boolean();
            }
        }

        Ok((alphabet, omit_padding))
    }

    fn create_set_from_result(
        read: JsValue,
        written: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Create { read, written } object
        let obj = JsObject::with_object_proto(context.intrinsics());
        obj.set(js_string!("read"), read, false, context)?;
        obj.set(js_string!("written"), written, false, context)?;
        Ok(obj.into())
    }
}
