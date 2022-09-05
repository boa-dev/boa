//! URI Handling Functions
//!
//! Uniform Resource Identifiers, or URIs, are Strings that identify resources (e.g. web pages or
//! files) and transport protocols by which to access them (e.g. HTTP or FTP) on the Internet. The
//! ECMAScript language itself does not provide any support for using URIs except for functions
//! that encode and decode URIs as described in 19.2.6.2, 19.2.6.3, 19.2.6.4 and 19.2.6.5
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-number-object

use super::BuiltIn;
use crate::{
    builtins::JsArgs, object::FunctionBuilder, property::Attribute, Context, JsResult, JsString,
    JsValue,
};

/// Constant with all the unescaped URI characters.
///
/// Contains `uriAlpha`, `DecimalDigit` and `uriMark`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: hhttps://tc39.es/ecma262/#prod-uriUnescaped
const URI_UNESCAPED: [u16; 69] = [
    // uriAlpha
    b'a' as u16,
    b'b' as u16,
    b'c' as u16,
    b'd' as u16,
    b'e' as u16,
    b'f' as u16,
    b'g' as u16,
    b'h' as u16,
    b'i' as u16,
    b'j' as u16,
    b'k' as u16,
    b'l' as u16,
    b'm' as u16,
    b'n' as u16,
    b'o' as u16,
    b'p' as u16,
    b'q' as u16,
    b'r' as u16,
    b's' as u16,
    b't' as u16,
    b'u' as u16,
    b'v' as u16,
    b'w' as u16,
    b'x' as u16,
    b'y' as u16,
    b'z' as u16,
    b'A' as u16,
    b'B' as u16,
    b'C' as u16,
    b'D' as u16,
    b'E' as u16,
    b'F' as u16,
    b'G' as u16,
    b'H' as u16,
    b'I' as u16,
    b'J' as u16,
    b'K' as u16,
    b'L' as u16,
    b'M' as u16,
    b'N' as u16,
    b'O' as u16,
    b'P' as u16,
    b'Q' as u16,
    b'R' as u16,
    b'S' as u16,
    b'T' as u16,
    b'U' as u16,
    b'V' as u16,
    b'W' as u16,
    b'X' as u16,
    b'Y' as u16,
    // DecimalDigit
    b'0' as u16,
    b'1' as u16,
    b'2' as u16,
    b'3' as u16,
    b'4' as u16,
    b'5' as u16,
    b'6' as u16,
    b'7' as u16,
    b'8' as u16,
    b'9' as u16,
    // uriMark
    b'-' as u16,
    b'_' as u16,
    b'.' as u16,
    b'!' as u16,
    b'~' as u16,
    b'*' as u16,
    b'\'' as u16,
    b'(' as u16,
];

/// URI Handling Functions
#[derive(Debug, Clone, Copy)]
pub(crate) struct Uri;

impl BuiltIn for Uri {
    const NAME: &'static str = "Number";

    fn init(context: &mut Context) -> Option<JsValue> {
        let decode_uri = FunctionBuilder::native(context, Self::decode_uri)
            .name("decodeURI")
            .length(1)
            .constructor(false)
            .build();

        context.register_global_property(
            "decodeURI",
            decode_uri,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let decode_uri_component = FunctionBuilder::native(context, Self::decode_uri_component)
            .name("decodeURIComponent")
            .length(1)
            .constructor(false)
            .build();

        context.register_global_property(
            "encodeURI",
            decode_uri_component,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let encode_uri = FunctionBuilder::native(context, Self::encode_uri)
            .name("encodeURI")
            .length(1)
            .constructor(false)
            .build();

        context.register_global_property(
            "encodeURI",
            encode_uri,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let encode_uri_component = FunctionBuilder::native(context, Self::encode_uri_component)
            .name("encodeURIComponent")
            .length(1)
            .constructor(false)
            .build();

        context.register_global_property(
            "encodeURIComponent",
            encode_uri_component,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        None
    }
}

impl Uri {
    /// Builtin JavaScript `decodeURI ( encodedURI )` function.
    ///
    /// This function computes a new version of a URI in which each escape sequence and UTF-8
    /// encoding of the sort that might be introduced by the `encodeURI` function is replaced with
    /// the UTF-16 encoding of the code points that it represents. Escape sequences that could not
    /// have been introduced by `encodeURI` are not replaced.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-decodeuri-encodeduri
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/decodeURI
    pub(crate) fn decode_uri(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let encoded_uri = args.get_or_undefined(0);

        // 1. Let uriString be ? ToString(encodedURI).
        let uri_string = encoded_uri.to_string(context)?;

        // 2. Let reservedURISet be a String containing one instance of each code unit valid in uriReserved plus "#".
        let reserved_uri_set = &URI_UNESCAPED; // TODO: add #

        // 3. Return ? Decode(uriString, reservedURISet).
        Ok(JsValue::from(decode(
            context,
            &uri_string,
            reserved_uri_set,
        )?))
    }

    /// Builtin JavaScript `decodeURIComponent ( encodedURIComponent )` function.
    ///
    /// This function computes a new version of a URI in which each escape sequence and UTF-8
    /// encoding of the sort that might be introduced by the `encodeURIComponent` function is
    /// replaced with the UTF-16 encoding of the code points that it represents.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-decodeuricomponent-encodeduricomponent
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/decodeURIComponent
    pub(crate) fn decode_uri_component(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let encoded_uri_component = args.get_or_undefined(0);

        // 1. Let componentString be ? ToString(encodedURIComponent).
        let component_string = encoded_uri_component.to_string(context)?;

        // 2. Let reservedURIComponentSet be the empty String.
        let reserved_uri_component_set = &[];

        // 3. Return ? Decode(componentString, reservedURIComponentSet).
        Ok(JsValue::from(decode(
            context,
            &component_string,
            reserved_uri_component_set,
        )?))
    }

    /// Builtin JavaScript `encodeURI ( uri )` function.
    ///
    /// This function computes a new version of a UTF-16 encoded (6.1.4) URI in which each instance
    /// of certain code points is replaced by one, two, three, or four escape sequences
    /// representing the UTF-8 encoding of the code points.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-encodeuri-uri
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURI
    pub(crate) fn encode_uri(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let uri = args.get_or_undefined(0);

        // 1. Let uriString be ? ToString(uri).
        let uri_string = uri.to_string(context)?;

        // 2. Let unescapedURISet be a String containing one instance of each code unit valid in uriReserved and uriUnescaped plus "#".
        let unescaped_uri_set = &URI_UNESCAPED; // TODO: add #

        // 3. Return ? Encode(uriString, unescapedURISet).
        Ok(JsValue::from(encode(
            context,
            &uri_string,
            unescaped_uri_set,
        )?))
    }

    /// Builtin JavaScript `encodeURIComponent ( uriComponent )` function.
    ///
    /// This function computes a new version of a UTF-16 encoded (6.1.4) URI in which each instance
    /// of certain code points is replaced by one, two, three, or four escape sequences
    /// representing the UTF-8 encoding of the code point.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-encodeuricomponent-uricomponent
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent
    pub(crate) fn encode_uri_component(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let uri_component = args.get_or_undefined(0);

        // 1. Let componentString be ? ToString(uriComponent).
        let component_string = uri_component.to_string(context)?;

        // 2. Let unescapedURIComponentSet be a String containing one instance of each code unit valid in uriUnescaped.
        let unescaped_uri_component_set = &URI_UNESCAPED;

        // 3. Return ? Encode(componentString, unescapedURIComponentSet).
        Ok(JsValue::from(encode(
            context,
            &component_string,
            unescaped_uri_component_set,
        )?))
    }
}

/// The `Encode ( string, unescapedSet )` abstract operation
///
/// The abstract operation Encode takes arguments `string` (a String) and `unescapedSet` (a String)
/// and returns either a normal completion containing a String or a throw completion. It performs
/// URI encoding and escaping.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-encode
fn encode(context: &mut Context, string: &JsString, unescaped_set: &[u16]) -> JsResult<String> {
    let code_units = string.encode_utf16().collect::<Vec<_>>();

    // 1. Let strLen be the length of string.
    let str_len = code_units.len();

    // 2. Let R be the empty String.
    let mut r = String::new();

    // 3. Let k be 0.
    let mut k = 0;
    // 4. Repeat,
    loop {
        // a. If k = strLen, return R.
        if k == str_len {
            return Ok(r);
        }

        // b. Let C be the code unit at index k within string.
        let c = code_units[k];

        // c. If C is in unescapedSet, then
        if unescaped_set.contains(&c) {
            // i. Set k to k + 1.
            k += 1;

            // ii. Set R to the string-concatenation of R and C.
            r.push(char::from_u32(u32::from(c)).expect("char from code point cannot fail here"));
        } else {
            // d. Else,
            // i. Let cp be CodePointAt(string, k).
            let cp = crate::builtins::string::code_point_at(string, k as u64);
            // ii. If cp.[[IsUnpairedSurrogate]] is true, throw a URIError exception.
            if cp.is_unpaired_surrogate {
                context.throw_uri_error("trying to encode an invalid string")?;
            }
            // iii. Set k to k + cp.[[CodeUnitCount]].
            k += cp.code_unit_count as usize;

            // iv. Let Octets be the List of octets resulting by applying the UTF-8 transformation
            //     to cp.[[CodePoint]].
            let mut buff = [0_u8; 4]; // Will never be more than 4 bytes

            let octets = char::from_u32(cp.code_point)
                .expect("valid unicode code point to char conversion failed")
                .encode_utf8(&mut buff);

            // v. For each element octet of Octets, do
            for octet in octets.bytes() {
                // 1. Set R to the string-concatenation of:
                //    R
                //    "%"
                //    the String representation of octet, formatted as a two-digit uppercase
                //    hexadecimal number, padded to the left with a zero if necessary
                r = format!("{r}%{octet:0>2X}");
            }
        }
    }
}

/// The `Decode ( string, reservedSet )` abstract operation.
///
/// The abstract operation Decode takes arguments `string` (a String) and `reservedSet` (a String)
/// and returns either a normal completion containing a String or a throw completion. It performs
/// URI unescaping and decoding.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-decode
#[allow(clippy::many_single_char_names)]
fn decode(context: &mut Context, string: &JsString, reserved_set: &[u16]) -> JsResult<String> {
    let code_units = string.encode_utf16().collect::<Vec<_>>();

    // 1. Let strLen be the length of string.
    let str_len = code_units.len();
    // 2. Let R be the empty String.
    let mut r = Vec::new();

    // 3. Let k be 0.
    let mut k = 0;
    // 4. Repeat,
    loop {
        // a. If k = strLen, return R.
        if k == str_len {
            return Ok(String::from_utf16(&r).expect("invalid UTF-16 characters found"));
        }

        // b. Let C be the code unit at index k within string.
        let c = code_units[k];

        // c. If C is not the code unit 0x0025 (PERCENT SIGN), then
        #[allow(clippy::if_not_else)]
        let s = if c != 0x0025_u16 {
            // i. Let S be the String value containing only the code unit C.
            vec![c]
        } else {
            // d. Else,
            // i. Let start be k.
            let start = k;

            // ii. If k + 2 ≥ strLen, throw a URIError exception.
            if k + 2 >= str_len {
                context.throw_uri_error("invalid escape character found")?;
            }

            // iii. If the code units at index (k + 1) and (k + 2) within string do not represent
            // hexadecimal digits, throw a URIError exception.
            let unit_k_1 = code_units[k + 1];
            let unit_k_2 = code_units[k + 2];
            if !is_hexdigit(unit_k_1) || !is_hexdigit(unit_k_2) {
                context.throw_uri_error("invalid escape character found")?;
            }

            // iv. Let B be the 8-bit value represented by the two hexadecimal digits at index (k + 1) and (k + 2).
            let b = decode_byte(unit_k_1, unit_k_2);

            // v. Set k to k + 2.
            k += 2;

            // vi. Let n be the number of leading 1 bits in B.
            let n = leading_one_bits(b);

            // vii. If n = 0, then
            if n == 0 {
                // 1. Let C be the code unit whose value is B.
                let c = u16::from(b);

                // 2. If C is not in reservedSet, then
                if !reserved_set.contains(&c) {
                    // a. Let S be the String value containing only the code unit C.
                    vec![c]
                } else {
                    // 3. Else,
                    // a. Let S be the substring of string from start to k + 1.
                    let mut s = Vec::new();
                    s.extend_from_slice(&code_units[start..k + 1]);
                    s
                }
            } else {
                // viii. Else,
                // 1. If n = 1 or n > 4, throw a URIError exception.
                if n == 1 || n > 4 {
                    context.throw_uri_error("TO-DO")?;
                }

                // 2. If k + (3 × (n - 1)) ≥ strLen, throw a URIError exception.
                if k + (3 * (n - 1)) > str_len {
                    context.throw_uri_error("TO-DO")?;
                }

                // 3. Let Octets be « B ».
                let mut octets = vec![b];

                // 4. Let j be 1.
                // 5. Repeat, while j < n,
                for _j in 1..n {
                    // a. Set k to k + 1.
                    k += 1;

                    // b. If the code unit at index k within string is not the code unit 0x0025 (PERCENT SIGN), throw a URIError exception.
                    if code_units[k] != 0x0025 {
                        context
                            .throw_uri_error("escape characters must be preceded with a % sign")?;
                    }

                    // c. If the code units at index (k + 1) and (k + 2) within string do not represent hexadecimal digits, throw a URIError exception.
                    let unit_k_1 = code_units[k + 1];
                    let unit_k_2 = code_units[k + 2];
                    if !is_hexdigit(unit_k_1) || !is_hexdigit(unit_k_2) {
                        context.throw_uri_error("invalid escape character")?;
                    }

                    // d. Let B be the 8-bit value represented by the two hexadecimal digits at index (k + 1) and (k + 2).
                    let b = decode_byte(unit_k_1, unit_k_2);

                    // e. Set k to k + 2.
                    k += 2;

                    // f. Append B to Octets.
                    octets.push(b);

                    // g. Set j to j + 1.
                }

                // 6. Assert: The length of Octets is n.
                assert_eq!(octets.len(), n);

                // 7. If Octets does not contain a valid UTF-8 encoding of a Unicode code point, throw a URIError exception.
                todo!();

                // 8. Let V be the code point obtained by applying the UTF-8 transformation to Octets, that is, from a List of octets into a 21-bit value.
                todo!();

                // 9. Let S be UTF16EncodeCodePoint(V).
                todo!()
            }
        };

        // e. Set R to the string-concatenation of R and S.
        r.extend_from_slice(&s);

        // f. Set k to k + 1.
        k += 1;
    }
}

/// Checks if a given code unit is an hexadecimal digit represented in UTF-16.
fn is_hexdigit(code_unit: u16) -> bool {
    use std::ops::RangeInclusive;

    const DIGIT: RangeInclusive<u16> = b'0' as u16..=b'9' as u16;
    const HEX_UPPER: RangeInclusive<u16> = b'A' as u16..=b'F' as u16;
    const HEX_LOWER: RangeInclusive<u16> = b'a' as u16..=b'f' as u16;

    DIGIT.contains(&code_unit) || HEX_UPPER.contains(&code_unit) || HEX_LOWER.contains(&code_unit)
}

/// Decodes a byte from two unicode code units. It expects both to be hexadecimal characters.
fn decode_byte(high: u16, low: u16) -> u8 {
    let high = high as u8 - b'0';
    let low = low as u8 - b'0';

    (high << 4) + low
}

/// Counts the number of leading 1 bits in a given byte.
fn leading_one_bits(byte: u8) -> usize {
    // This uses a value table for speed
    if byte == u8::MAX {
        8
    } else if byte == 0b1111_1110 {
        7
    } else if byte & 0b1111_1100 == 0b1111_1100 {
        6
    } else if byte & 0b1111_1000 == 0b1111_1000 {
        5
    } else if byte & 0b1111_0000 == 0b1111_0000 {
        4
    } else if byte & 0b1110_0000 == 0b1110_0000 {
        3
    } else if byte & 0b1100_1100 == 0b1100_0000 {
        2
    } else if byte & 0b1000_0000 == 0b1000_0000 {
        1
    } else {
        0
    }
}
