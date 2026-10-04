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

    pub(crate) fn decode(mut input: &[u8], strip_bom: bool, fatal: bool) -> Result<JsString, ()> {
        if strip_bom {
            input = input.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(input);
        }
        if fatal {
            std::str::from_utf8(input)
                .map(JsString::from)
                .map_err(|_| ())
        } else {
            let string = String::from_utf8_lossy(input);
            Ok(JsString::from(string.as_ref()))
        }
    }
}

/// Decodes an iterator of UTF-16 code units into a `JsString`.
///
/// If `fatal` is true, encountering an unpaired surrogate or a dangling (odd) byte
/// returns `Err(())`.
/// If `fatal` is false, unpaired surrogates and dangling bytes are replaced with `\u{FFFD}`.
fn decode_utf16_units(
    code_units: impl IntoIterator<Item = u16>,
    dangling_byte: bool,
    fatal: bool,
) -> Result<boa_engine::JsString, ()> {
    let mut string = String::new();
    let mut last_code_unit = None;
    for result in std::char::decode_utf16(
        code_units
            .into_iter()
            .inspect(|&u| last_code_unit = Some(u)),
    ) {
        match result {
            Ok(c) => string.push(c),
            Err(_) if fatal => return Err(()),
            Err(_) => string.push('\u{FFFD}'),
        }
    }
    if dangling_byte {
        if fatal {
            return Err(());
        }
        let trailing_high_surrogate =
            last_code_unit.is_some_and(|code_unit| (0xD800..=0xDBFF).contains(&code_unit));
        // At EOF, the Encoding Standard reports one error for a pending high
        // surrogate and an incomplete code unit together. decode_utf16 has
        // already emitted that surrogate's replacement character.
        if !trailing_high_surrogate {
            string.push('\u{FFFD}');
        }
    }
    Ok(boa_engine::JsString::from(string))
}

pub(crate) mod utf16le {
    use boa_engine::JsString;

    pub(crate) fn decode(mut input: &[u8], strip_bom: bool, fatal: bool) -> Result<JsString, ()> {
        if strip_bom {
            input = input.strip_prefix(&[0xFF, 0xFE]).unwrap_or(input);
        }

        let (pairs, remainder) = input.as_chunks::<2>();
        let code_units = pairs.iter().copied().map(u16::from_le_bytes);

        super::decode_utf16_units(code_units, !remainder.is_empty(), fatal)
    }
}

pub(crate) mod utf16be {
    use boa_engine::JsString;

    pub(crate) fn decode(mut input: &[u8], strip_bom: bool, fatal: bool) -> Result<JsString, ()> {
        if strip_bom {
            input = input.strip_prefix(&[0xFE, 0xFF]).unwrap_or(input);
        }

        let (pairs, remainder) = input.as_chunks::<2>();
        let code_units = pairs.iter().copied().map(u16::from_be_bytes);

        super::decode_utf16_units(code_units, !remainder.is_empty(), fatal)
    }
}
