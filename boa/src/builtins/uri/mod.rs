use crate::builtins::BuiltIn;
use crate::property::Attribute;
use crate::{Context, JsValue, JsResult, JsString};
use crate::object::function::make_builtin_fn;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use crate::builtins::string::{code_point_at, is_leading_surrogate, is_trailing_surrogate};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

fn is_uri_mark(code_point: u16) -> bool {
    [
        "-",
        "_",
        ".",
        "!",
        "~",
        "*",
        "'",
        "(",
        ")"
    ]
        .map(|glyphs| { glyphs.encode_utf16().next().unwrap() })
        .contains(&code_point)
}

fn is_uri_reserved(code_point: u16) -> bool {
    [
        ";",
        "/",
        "?",
        ":",
        "@",
        "&",
        "=",
        "+",
        "$",
        ","
    ]
        .map(|glyphs| { glyphs.encode_utf16().next().unwrap() })
        .contains(&code_point)
}

fn is_alpha_numeric(code_point: u16) -> bool {
    return (97..=122).contains(&code_point)
        || (65..=90).contains(&code_point)
        || (48..=57).contains(&code_point);
}

fn is_unescaped_uri_component_character(code_point: u16) -> bool {
    return is_alpha_numeric(code_point) || is_uri_mark(code_point);
}

fn naive_decimal_to_hexadecimal(number: u32) -> String {
    fn to_char(decimal_digit: usize) -> String {
        let alpha = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"];
        return alpha[decimal_digit].clone().to_string();
    }

    let mut remaining = number.clone();
    let mut r = &number % 16;
    let mut hex_number = "".to_string();

    while &remaining - &r != 0 {
        hex_number = to_char(r as usize) + &hex_number[..];
        remaining = (remaining - r) / 16;
        r = &remaining % 16;
    }

    return to_char(r as usize) + &hex_number[..];
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uri;

impl BuiltIn for Uri {
    const NAME: &'static str = "Uri";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let global = context.global_object();

        make_builtin_fn(
            Self::encode_uri,
            "encodeURI",
            &global,
            1,
            context,
        );

        make_builtin_fn(
            Self::encode_uri_component,
            "encodeURIComponent",
            &global,
            1,
            context,
        );

        (Self::NAME, JsValue::undefined(), Self::attribute())
    }
}

fn utf8_encode(str: &mut Vec<u32>, code_point: u32, replace_invalid: bool) {
    //unsigned Utf8::Encode(char* str, uchar c, int previous, bool replace_invalid) {
    let k_mask = !(1 << 6);
    if code_point <= 0x007F {
        str.push(code_point);
    } else if code_point <= 0x07FF {
        str.push(0xC0 | (code_point >> 6));
        str.push(0x80 | (code_point & k_mask));
    } else if code_point <= 0xFFFF {
        // DCHECK(!Utf16::IsLeadSurrogate(Utf16::kNoPreviousCharacter));
        // if (Utf16::IsSurrogatePair(previous, code_point)) {
        //     const int kUnmatchedSize = kSizeOfUnmatchedSurrogate;
        //     return Encode(str - kUnmatchedSize,
        //                   Utf16::CombineSurrogatePair(previous, code_point),
        //                   Utf16::kNoPreviousCharacter, replace_invalid) -
        //         kUnmatchedSize;
        // } else if (replace_invalid &&
        //     (Utf16::IsLeadSurrogate(code_point) || Utf16::IsTrailSurrogate(code_point))) {
        //     code_point = kBadChar;
        // }
        str.push(0xE0 | (code_point >> 12));
        str.push(0x80 | ((code_point >> 6) & k_mask));
        str.push(0x80 | (code_point & k_mask));
    } else {
        str.push(0xF0 | (code_point >> 18));
        str.push(0x80 | ((code_point >> 12) & k_mask));
        str.push(0x80 | ((code_point >> 6) & k_mask));
        str.push(0x80 | (code_point & k_mask));
    }
}

fn add_encoded_octet_to_buffer(utf8_encoded: &u32, encoded_result: &mut String) {
    let value1 = String::from(naive_decimal_to_hexadecimal((utf8_encoded >> 4)));
    let value2 = String::from(naive_decimal_to_hexadecimal((utf8_encoded & 0x0F)));

    encoded_result.push_str("%");
    encoded_result.push_str(value1.as_str());
    encoded_result.push_str(value2.as_str());
}

fn encode_single(code_point: u32, encoded_result: &mut String) {
    let mut utf8_encoded = Vec::<u32>::new();
    utf8_encode(&mut utf8_encoded, code_point, false);

    utf8_encoded.iter()
        .for_each(|encoded| { add_encoded_octet_to_buffer(encoded, encoded_result) });
}

impl Uri {
    fn encode(string: JsString, is_uri_component: bool) -> String {
        println!("encode(): {:?}", &string);
        let mut encoded_result = "".to_string();

        string.encode_utf16()
            .map(|code_point| { code_point as u32 })
            .for_each(|code_point: u32| {
                println!("encode(): code_point = {}", &code_point);

                if is_leading_surrogate(code_point as u16) {
                    panic!("is leading surrogate");
                    // let next_code_point = encoded.next();
                    // if let Some(code_point_pair) = next_code_point {
                    //     if is_trailing_surrogate(code_point_pair) {
                    //         encode_pair(code_point, next_code_point, &mut encoded_result);
                    //     }
                    // }
                } else if !is_trailing_surrogate(code_point as u16) {
                    if is_unescaped_uri_component_character(code_point as u16)
                        || (!is_uri_component && is_uri_reserved(code_point as u16)) {

                        if let Ok(value) = String::from_utf16(&[code_point as u16]) {
                            encoded_result.push_str(&value[..]);
                        } else {
                            panic!("encode(): failure1");
                        }
                    } else {
                        encode_single(code_point, &mut encoded_result);
                    }
                } else {
                    panic!("URIError");
                }
            });

        encoded_result
    }

    pub(crate) fn encode_uri(
        _: &JsValue,
        args: &[JsValue],
        _ctx: &mut Context,
    ) -> JsResult<JsValue> {
        Ok(JsValue::new(if let Some(value) = args.get(0) {
            match value {
                JsValue::String(string_value) => utf8_percent_encode(string_value, FRAGMENT).to_string(),
                _ => "undefined1".to_string()
            }
        } else {
            "undefined".to_string()
        }))
    }

    pub(crate) fn encode_uri_component(
        _: &JsValue,
        args: &[JsValue],
        ctx: &mut Context,
    ) -> JsResult<JsValue> {
        Ok(JsValue::new(if let Some(value) = args.get(0) {
            Uri::encode(value.to_string(ctx).unwrap(), true)
        } else {
            "undefined".to_string()
        }))
    }
}
