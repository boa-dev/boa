pub(crate) mod utf8 {
    use boa_engine::string::CodePoint;
    use boa_engine::{JsResult, JsString, js_error};

    pub(crate) fn encode(input: &JsString) -> Vec<u8> {
        input
            .code_points()
            .flat_map(|s| match s {
                CodePoint::Unicode(c) => c.to_string().as_bytes().to_vec(),
                CodePoint::UnpairedSurrogate(_) => "\u{FFFD}".as_bytes().to_vec(),
            })
            .collect()
    }

    pub(crate) fn decode(input: &[u8], fatal: bool) -> JsResult<JsString> {
        if fatal {
            let s = std::str::from_utf8(input)
                .map_err(|_| js_error!(TypeError: "The encoded data was not valid."))?;
            Ok(JsString::from(s))
        } else {
            let string = String::from_utf8_lossy(input);
            Ok(JsString::from(string.as_ref()))
        }
    }
}

pub(crate) mod utf16le {
    use boa_engine::string::JsStrVariant;
    use boa_engine::{JsResult, JsString, js_error, js_string};

    pub(crate) fn encode(input: &JsString) -> Vec<u8> {
        match input.as_str().variant() {
            JsStrVariant::Latin1(l) => l.iter().flat_map(|c| [*c, 0]).collect(),
            JsStrVariant::Utf16(s) => bytemuck::cast_slice(s).to_vec(),
        }
    }

    pub(crate) fn decode(mut input: &[u8], fatal: bool) -> JsResult<JsString> {
        let dangling = if input.len().is_multiple_of(2) {
            false
        } else {
            if fatal {
                return Err(js_error!(TypeError: "The encoded data was not valid."));
            }
            input = &input[0..input.len() - 1];
            true
        };

        let input: &[u16] = bytemuck::cast_slice(input);

        if fatal {
            let s = String::from_utf16(input)
                .map_err(|_| js_error!(TypeError: "The encoded data was not valid."))?;
            Ok(JsString::from(s))
        } else if dangling {
            Ok(JsString::from(&[
                JsString::from(input),
                js_string!("\u{FFFD}"),
            ]))
        } else {
            Ok(JsString::from(input))
        }
    }
}

pub(crate) mod utf16be {
    use boa_engine::string::JsStrVariant;
    use boa_engine::{JsResult, JsString, js_error, js_string};

    pub(crate) fn encode(input: &JsString) -> Vec<u8> {
        match input.as_str().variant() {
            JsStrVariant::Latin1(l) => l.iter().flat_map(|c| [0, *c]).collect(),
            JsStrVariant::Utf16(s) => s.iter().flat_map(|b| b.to_be_bytes()).collect::<Vec<_>>(),
        }
    }

    pub(crate) fn decode(mut input: Vec<u8>, fatal: bool) -> JsResult<JsString> {
        let mut input = input.as_mut_slice();
        let dangling = if input.len().is_multiple_of(2) {
            false
        } else {
            if fatal {
                return Err(js_error!(TypeError: "The encoded data was not valid."));
            }
            let new_len = input.len() - 1;
            input = &mut input[0..new_len];
            true
        };

        let input: &mut [u16] = bytemuck::cast_slice_mut(input);

        for b in &mut *input {
            *b = b.swap_bytes();
        }

        if fatal {
            let s = String::from_utf16(input)
                .map_err(|_| js_error!(TypeError: "The encoded data was not valid."))?;
            Ok(JsString::from(s))
        } else if dangling {
            Ok(JsString::from(&[
                JsString::from(&*input),
                js_string!("\u{FFFD}"),
            ]))
        } else {
            Ok(JsString::from(&*input))
        }
    }
}
