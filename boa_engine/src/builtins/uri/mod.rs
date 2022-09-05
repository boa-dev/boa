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

mod consts;

use super::BuiltIn;
use crate::{
    builtins::JsArgs, object::FunctionBuilder, property::Attribute, Context, JsResult, JsString,
    JsValue,
};
use consts::*;

/// URI Handling Functions
#[derive(Debug, Clone, Copy)]
pub(crate) struct Uri;

impl BuiltIn for Uri {
    const NAME: &'static str = "Uri";

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
            "decodeURIComponent",
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
        let reserved_uri_set = &URI_RESERVED_HASH;

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
        let unescaped_uri_set = &URI_RESERVED_UNESCAPED_HASH;

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
                    s.extend_from_slice(&code_units[start..=k]);
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
                match String::from_utf8(octets) {
                    Err(_) => {
                        return Err(context.construct_uri_error("invalid UTF-8 encoding found"))
                    }
                    Ok(v) => {
                        // 8. Let V be the code point obtained by applying the UTF-8 transformation to Octets, that is, from a List of octets into a 21-bit value.

                        // 9. Let S be UTF16EncodeCodePoint(V).
                        // utf16_encode_codepoint(v)
                        v.encode_utf16().collect::<Vec<_>>()
                    }
                }
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
    let high = u8::try_from(high).expect("invalid ASCII character found");
    let low = u8::try_from(low).expect("invalid ASCII character found");

    let high = if (b'0'..=b'9').contains(&high) {
        high - b'0'
    } else if (b'A'..=b'Z').contains(&high) {
        high - b'A' + 0x0A
    } else if (b'a'..=b'z').contains(&high) {
        high - b'a' + 0x0A
    } else {
        panic!("invalid ASCII hexadecimal digit found");
    };

    let low = if (b'0'..=b'9').contains(&low) {
        low - b'0'
    } else if (b'A'..=b'Z').contains(&low) {
        low - b'A' + 0x0A
    } else if (b'a'..=b'z').contains(&low) {
        low - b'a' + 0x0A
    } else {
        panic!("invalid ASCII hexadecimal digit found");
    };

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
    } else if byte & 0b1100_0000 == 0b1100_0000 {
        2
    } else if byte & 0b1000_0000 == 0b1000_0000 {
        1
    } else {
        0
    }
}

// /// Generates the string representation of
// fn utf16_encode_codepoint(cp: u16) -> String {
//     // 1. Assert: 0 ≤ cp ≤ 0x10FFFF.
//     assert!(cp <= 0x10FFFF);

//     // 2. If cp ≤ 0xFFFF, return the String value consisting of the code unit whose value is cp.
//     if cp <= 0xFFFF {
//         return String::from_utf16(&[cp]).expect("invalid UTF-16 code units found");
//     }

//     // 3. Let cu1 be the code unit whose value is floor((cp - 0x10000) / 0x400) + 0xD800.
//     let cu1 = ((cp - 0x10000) as f64 / 0x400 as f64).floor() as u16 + 0xD800;

//     // 4. Let cu2 be the code unit whose value is ((cp - 0x10000) modulo 0x400) + 0xDC00.
//     let cu2 = ((cp - 0x10000) % 0x400) + 0xD800;

//     // 5. Return the string-concatenation of cu1 and cu2.
//     String::from_utf16(&[cu1, cu2]).expect("invalid UTF-16 code units found")
// }

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks if the `leading_one_bits()` function works as expected.
    #[test]
    fn ut_leading_one_bits() {
        assert_eq!(leading_one_bits(0b1111_1111), 8);
        assert_eq!(leading_one_bits(0b1111_1110), 7);

        assert_eq!(leading_one_bits(0b1111_1100), 6);
        assert_eq!(leading_one_bits(0b1111_1101), 6);

        assert_eq!(leading_one_bits(0b1111_1011), 5);
        assert_eq!(leading_one_bits(0b1111_1000), 5);

        assert_eq!(leading_one_bits(0b1111_0000), 4);
        assert_eq!(leading_one_bits(0b1111_0111), 4);

        assert_eq!(leading_one_bits(0b1110_0000), 3);
        assert_eq!(leading_one_bits(0b1110_1111), 3);

        assert_eq!(leading_one_bits(0b1100_0000), 2);
        assert_eq!(leading_one_bits(0b1101_1111), 2);

        assert_eq!(leading_one_bits(0b1000_0000), 1);
        assert_eq!(leading_one_bits(0b1011_1111), 1);

        assert_eq!(leading_one_bits(0b0000_0000), 0);
        assert_eq!(leading_one_bits(0b0111_1111), 0);
    }

    /// Checks that the `decode_byte()` function works as expected.
    #[test]
    fn ut_decode_byte() {
        assert_eq!(decode_byte(u16::from(b'2'), u16::from(b'0')), 0x20);
        assert_eq!(decode_byte(u16::from(b'2'), u16::from(b'A')), 0x2A);
        assert_eq!(decode_byte(u16::from(b'3'), u16::from(b'C')), 0x3C);
        assert_eq!(decode_byte(u16::from(b'4'), u16::from(b'0')), 0x40);
        assert_eq!(decode_byte(u16::from(b'7'), u16::from(b'E')), 0x7E);
        assert_eq!(decode_byte(u16::from(b'0'), u16::from(b'0')), 0x00);
    }

    /// Checks that the `decode_byte()` panics with invalid ASCII characters.
    #[test]
    #[should_panic]
    fn ut_decode_byte_rainy() {
        decode_byte(u16::from(b'-'), u16::from(b'0'));
    }
}
