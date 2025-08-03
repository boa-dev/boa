pub(crate) mod utf8 {
    use boa_engine::JsString;
    use boa_engine::string::CodePoint;

    pub(crate) fn encode(input: &JsString) -> Vec<u8> {
        input
            .code_points()
            .flat_map(|s| match s {
                CodePoint::Unicode(c) => c.to_string().as_bytes().to_vec(),
                CodePoint::UnpairedSurrogate(_) => "\u{FFFD}".as_bytes().to_vec(),
            })
            .collect()
    }

    pub(crate) fn decode(input: &[u8]) -> JsString {
        let string = String::from_utf8_lossy(input);
        JsString::from(string.as_ref())
    }
}

pub(crate) mod utf16le {
    use boa_engine::{JsString, js_string};

    pub(crate) fn encode(input: &JsString) -> Vec<u8> {
        let bytes = input.as_str().to_vec();
        let bytes = bytes.as_slice();
        bytemuck::cast_slice(bytes).to_vec()
    }

    pub(crate) fn decode(mut input: &[u8]) -> JsString {
        // After this point, input is of even length.
        let dangling = if input.len() % 2 != 0 {
            input = &input[0..input.len() - 1];
            true
        } else {
            false
        };

        let input: &[u16] = bytemuck::cast_slice(input);

        if dangling {
            JsString::from(&[JsString::from(input), js_string!("\u{fffd}")])
        } else {
            JsString::from(input)
        }
    }
}

pub(crate) mod utf16be {
    use boa_engine::{JsString, js_string};

    pub(crate) fn encode(input: &JsString) -> Vec<u8> {
        let mut bytes = input.as_str().to_vec();

        // Swap the bytes.
        for b in bytes.as_mut_slice() {
            *b = *b >> 8 | (*b & 0xFF) << 8;
        }

        bytemuck::cast_slice(bytes.as_mut_slice()).to_vec()
    }

    pub(crate) fn decode(mut input: Vec<u8>) -> JsString {
        let mut input = input.as_mut_slice();
        // After this point, input is of even length.
        let dangling = if input.len() % 2 != 0 {
            let new_len = input.len() - 1;
            input = &mut input[0..new_len];
            true
        } else {
            false
        };

        let input: &mut [u16] = bytemuck::cast_slice_mut(input);

        // Swap the bytes.
        for b in &mut *input {
            *b = *b >> 8 | (*b & 0xFF) << 8;
        }

        if dangling {
            JsString::from(&[JsString::from(&*input), js_string!("\u{fffd}")])
        } else {
            JsString::from(&*input)
        }
    }
}
