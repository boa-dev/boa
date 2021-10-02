use crate::builtins::BuiltIn;
use crate::property::Attribute;
use crate::{Context, JsValue, JsResult, JsString};
use crate::object::function::make_builtin_fn;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use crate::builtins::string::code_point_at;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

fn is_uri_mark(c: &str) -> bool {
    return match c {
        "-" => true,
        "_" => true,
        "." => true,
        "!" => true,
        "~" => true,
        "*" => true,
        "'" => true,
        "(" => true,
        ")" => true,
        _ => false
    }
}

fn is_uri_reserved(c: &str) -> bool {
    return match c {
        ";" => true,
        "/" => true,
        "?" => true,
        ":" => true,
        "@" => true,
        "&" => true,
        "=" => true,
        "+" => true,
        "$" => true,
        "," => true,
        _ => false
    }
}

fn is_alpha_numeric(c: &str) -> bool {
    let first_byte = c.as_bytes()[0];
    return (97..=122).contains(&first_byte)
        || (65..=90).contains(&first_byte)
        || (48..=57).contains(&first_byte);
}

fn is_unescaped_uri_component_character(c: &str) -> bool {
    return is_alpha_numeric(&c) || is_uri_mark(&c);
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

impl Uri {
    fn encode(string: JsString, is_uri_component: bool) -> String {
        let str_len = (&string).len();
        let mut r = "".to_string();
        let mut k: usize = 0;

        loop {
            if str_len == k {
                return r;
            }

            let c = code_point_at(string.clone(), k as i32);
            if let Some((code_point, code_point_count, is_unpaired_surrogate)) = c {
                let character = &String::from_utf8(vec![code_point as u8]).unwrap()[..];

                let should_push_unencoded = is_unescaped_uri_component_character(character)
                    || (!is_uri_component && is_uri_reserved(character));

                if should_push_unencoded {
                    k += 1;
                    r.push_str(character);
                } else {
                    // TODO: return URIError if code point is IsUnpairedSurrogate
                    if is_unpaired_surrogate {
                        panic!("encode uri failed: unpaired surrogate found");
                    }

                    k += code_point_count as usize;
                    let utf8_hex = naive_decimal_to_hexadecimal(code_point);
                    r.push_str("%");
                    r.push_str(&utf8_hex[..]);
                }
            }
        }
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
