use boa_engine::string::CodePoint;
use boa_engine::{Context, JsResult, JsString, js_string};
use std::rc::Rc;

pub(super) trait Encoder: std::fmt::Debug {
    fn name(&self) -> JsString;
    fn encode(&self, input: &JsString, context: &Context) -> JsResult<Vec<u8>>;
    fn decode(&self, input: Vec<u8>, context: &Context) -> JsResult<JsString>;
}

#[derive(Debug)]
struct Utf8Encoder;
impl Encoder for Utf8Encoder {
    fn name(&self) -> JsString {
        js_string!("utf-8")
    }
    fn encode(&self, input: &JsString, _context: &Context) -> JsResult<Vec<u8>> {
        Ok(input
            .code_points()
            .flat_map(|s| match s {
                CodePoint::Unicode(c) => c.to_string().as_bytes().to_vec(),
                CodePoint::UnpairedSurrogate(_) => "\u{FFFD}".as_bytes().to_vec(),
            })
            .collect())
    }

    fn decode(&self, input: Vec<u8>, _context: &Context) -> JsResult<JsString> {
        let string = String::from_utf8_lossy(input.as_slice());
        Ok(JsString::from(string.as_ref()))
    }
}

#[derive(Debug)]
struct Utf16LeEncoder;
impl Encoder for Utf16LeEncoder {
    fn name(&self) -> JsString {
        js_string!("utf-16le")
    }
    fn encode(&self, input: &JsString, _context: &Context) -> JsResult<Vec<u8>> {
        let bytes = input.as_str().to_vec();
        let len = bytes.len() * 2;

        // SAFETY: The vector is u16, and we transmute to u8, so this will never fail
        // nor have any misalignment.
        Ok(unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast::<u8>(), len).to_vec() })
    }

    fn decode(&self, input: Vec<u8>, _context: &Context) -> JsResult<JsString> {
        let mut input = input.as_slice();
        // After this point, input is of even length.
        let dangling = if input.len() % 2 != 0 {
            input = &input[0..input.len() - 1];
            true
        } else {
            false
        };

        let len = input.len() / 2;

        // Transmute into &[u16].
        // SAFETY: This is safe as any dangling bytes have been removed from the slice,
        // and the len is correct.
        #[allow(clippy::cast_ptr_alignment)]
        let input = unsafe { std::slice::from_raw_parts(input.as_ptr().cast::<u16>(), len) };

        if dangling {
            Ok(JsString::from(&[
                JsString::from(input),
                js_string!("\u{fffd}"),
            ]))
        } else {
            Ok(JsString::from(input))
        }
    }
}

#[derive(Debug)]
struct Utf16BeEncoder;
impl Encoder for Utf16BeEncoder {
    fn name(&self) -> JsString {
        js_string!("utf-16be")
    }
    fn encode(&self, input: &JsString, _context: &Context) -> JsResult<Vec<u8>> {
        let mut bytes = input.as_str().to_vec();
        let len = bytes.len() * 2;

        // Swap the bytes.
        for b in bytes.as_mut_slice() {
            *b = *b >> 8 | (*b & 0xFF) << 8;
        }

        // SAFETY: The vector is u16, and we transmute to u8, so this will never fail
        // nor have any misalignment.
        Ok(unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast::<u8>(), len).to_vec() })
    }

    fn decode(&self, mut input: Vec<u8>, _context: &Context) -> JsResult<JsString> {
        let mut input = input.as_mut_slice();
        // After this point, input is of even length.
        let dangling = if input.len() % 2 != 0 {
            let new_len = input.len() - 1;
            input = &mut input[0..new_len];
            true
        } else {
            false
        };

        let len = input.len() / 2;

        // Transmute into &[u16].
        // SAFETY: This is safe as any dangling bytes have been removed from the slice,
        // and the len is correct.
        #[allow(clippy::cast_ptr_alignment)]
        let input =
            unsafe { std::slice::from_raw_parts_mut(input.as_mut_ptr().cast::<u16>(), len) };

        // Swap the bytes.
        for b in &mut *input {
            *b = *b >> 8 | (*b & 0xFF) << 8;
        }

        if dangling {
            Ok(JsString::from(&[
                JsString::from(&*input),
                js_string!("\u{fffd}"),
            ]))
        } else {
            Ok(JsString::from(&*input))
        }
    }
}

pub(super) fn create_encoder(encoding: &JsString) -> Option<Rc<dyn Encoder>> {
    match encoding.to_std_string_lossy().as_str() {
        "utf-8" => Some(Rc::new(Utf8Encoder)),
        // Default encoding is Little Endian.
        "utf-16" | "utf-16le" => Some(Rc::new(Utf16LeEncoder)),
        "utf-16be" => Some(Rc::new(Utf16BeEncoder)),
        _ => None,
    }
}
