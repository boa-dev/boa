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

/// Decodes an iterator of UTF-16 code units into a well-formed `JsString`,
/// replacing any unpaired surrogates with U+FFFD.
///
/// If `dangling_byte` is true and the last decoded code unit is not a high
/// surrogate (which would already have been replaced), an additional U+FFFD
/// is appended for the truncated trailing byte.
fn decode_utf16_units(
    code_units: impl IntoIterator<Item = u16>,
    dangling_byte: bool,
) -> boa_engine::JsString {
    let mut string = String::new();
    let mut last_code_unit = None;
    string.extend(
        std::char::decode_utf16(code_units.into_iter().inspect(|code_unit| {
            last_code_unit = Some(*code_unit);
        }))
        .map(|result| result.unwrap_or('\u{FFFD}')),
    );
    let trailing_high_surrogate =
        last_code_unit.is_some_and(|code_unit| (0xD800..=0xDBFF).contains(&code_unit));
    if dangling_byte && !trailing_high_surrogate {
        string.push('\u{FFFD}');
    }
    boa_engine::JsString::from(string)
}

pub(crate) mod utf16le {
    use boa_engine::JsString;

    pub(crate) fn decode(mut input: &[u8], strip_bom: bool) -> JsString {
        if strip_bom {
            input = input.strip_prefix(&[0xFF, 0xFE]).unwrap_or(input);
        }

        let dangling_byte = !input.len().is_multiple_of(2);
        if dangling_byte {
            input = &input[0..input.len() - 1];
        }

        let code_units: &[u16] = bytemuck::cast_slice(input);
        super::decode_utf16_units(code_units.iter().copied(), dangling_byte)
    }
}

pub(crate) mod utf16be {
    use boa_engine::JsString;

    pub(crate) fn decode(mut input: &[u8], strip_bom: bool) -> JsString {
        if strip_bom && let Some(rest) = input.strip_prefix(&[0xFE, 0xFF]) {
            input = rest;
        }

        let dangling_byte = !input.len().is_multiple_of(2);
        if dangling_byte {
            input = &input[0..input.len() - 1];
        }

        let code_units = input
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]));
        super::decode_utf16_units(code_units, dangling_byte)
    }
}
