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

    pub(crate) fn decode(mut input: &[u8], strip_bom: bool) -> JsString {
        if strip_bom {
            input = input.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(input);
        }
        let string = String::from_utf8_lossy(input);
        JsString::from(string.as_ref())
    }
}

pub(crate) mod utf16le {
    use boa_engine::string::JsStrVariant;
    use boa_engine::{JsString, js_string};

    pub(crate) fn encode(input: &JsString) -> Vec<u8> {
        match input.as_str().variant() {
            JsStrVariant::Latin1(l) => l.iter().flat_map(|c| [*c, 0]).collect(),
            JsStrVariant::Utf16(s) => bytemuck::cast_slice(s).to_vec(),
        }
    }

    pub(crate) fn decode(mut input: &[u8], strip_bom: bool) -> JsString {
        if strip_bom {
            input = input.strip_prefix(&[0xFF, 0xFE]).unwrap_or(input);
        }

        // After this point, input is of even length.
        let dangling = if input.len().is_multiple_of(2) {
            false
        } else {
            input = &input[0..input.len() - 1];
            true
        };

        let input: &[u16] = bytemuck::cast_slice(input);

        if dangling {
            JsString::from(&[JsString::from(input), js_string!("\u{FFFD}")])
        } else {
            JsString::from(input)
        }
    }
}

pub(crate) mod utf16be {
    use boa_engine::string::JsStrVariant;
    use boa_engine::{JsString, js_string};

    pub(crate) fn encode(input: &JsString) -> Vec<u8> {
        match input.as_str().variant() {
            JsStrVariant::Latin1(l) => l.iter().flat_map(|c| [0, *c]).collect(),
            JsStrVariant::Utf16(s) => s.iter().flat_map(|b| b.to_be_bytes()).collect::<Vec<_>>(),
        }
    }

    pub(crate) fn decode(mut input: Vec<u8>, strip_bom: bool) -> JsString {
        if strip_bom && input.starts_with(&[0xFE, 0xFF]) {
            input.drain(..2);
        }

        let mut input = input.as_mut_slice();
        // After this point, input is of even length.
        let dangling = if input.len().is_multiple_of(2) {
            false
        } else {
            let new_len = input.len() - 1;
            input = &mut input[0..new_len];
            true
        };

        let input: &mut [u16] = bytemuck::cast_slice_mut(input);

        // Swap the bytes.
        for b in &mut *input {
            *b = b.swap_bytes();
        }

        if dangling {
            JsString::from(&[JsString::from(&*input), js_string!("\u{FFFD}")])
        } else {
            JsString::from(&*input)
        }
    }
}
