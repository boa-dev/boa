//! This module implements the global `Number` object.
//!
//! The `Number` JavaScript object is a wrapper object allowing you to work with numerical values.
//! A `Number` object is created using the `Number()` constructor. A primitive type object number is created using the `Number()` **function**.
//!
//! The JavaScript `Number` type is double-precision 64-bit binary format IEEE 754 value. In more recent implementations,
//! JavaScript also supports integers with arbitrary precision using the BigInt type.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-number-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number

use super::function::make_builtin_fn;
use std::borrow::Borrow;

use crate::{
    builtins::BuiltIn, object::ConstructorBuilder, property::Attribute, value::Value, BoaProfiler,
    Context, Result,
};

use percent_encoding::{percent_decode, utf8_percent_encode, AsciiSet, CONTROLS};

// https://url.spec.whatwg.org/#fragment-percent-encode-set
const ENCODE_FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
const PARSE_INT_MAX_ARG_COUNT: usize = 1;

#[derive(Debug, Clone, Copy)]
pub(crate) struct EncodeURI;

impl BuiltIn for EncodeURI {
    const NAME: &'static str = "encodeURI";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT;
        let number_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().number_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .build();

        let global = context.global_object();
        make_builtin_fn(
            Self::decode_uri,
            "decodeURI",
            &global,
            PARSE_INT_MAX_ARG_COUNT,
            context,
        );

        (Self::NAME, number_object.into(), Self::attribute())
    }
}

impl EncodeURI {
    pub(crate) const LENGTH: usize = 1;

    pub(crate) fn constructor(
        new_target: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        let first_arg = match args.get(0) {
            Some(Value::String(ref str)) => {
                if str.len() == 0 {
                    Value::string("")
                } else {
                    let encoded = utf8_percent_encode(str, ENCODE_FRAGMENT).to_string();
                    Value::string(encoded)
                }
            }
            _ => Value::Undefined,
        };

        Ok(first_arg)
    }

    pub(crate) fn decode_uri(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let first_arg = match args.get(0) {
            Some(Value::String(ref str)) => {
                if str.len() == 0 {
                    Value::string("")
                } else {
                    let encoded = percent_decode(str.as_bytes()).decode_utf8().unwrap();
                    Value::string(encoded.borrow())
                }
            }
            _ => Value::Undefined,
        };

        Ok(first_arg)
    }
}
