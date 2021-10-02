use crate::builtins::BuiltIn;
use crate::property::Attribute;
use crate::{Context, JsValue, JsResult};
use crate::object::function::make_builtin_fn;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

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

        (Self::NAME, JsValue::undefined(), Self::attribute())
    }
}

impl Uri {
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
}
