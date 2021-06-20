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

use std::borrow::Borrow;

use crate::value::RcString;
use crate::{
    builtins::BuiltIn, object::FunctionBuilder, property::Attribute, value::Value, BoaProfiler,
    Context, Result,
};
use percent_encoding::{percent_decode, utf8_percent_encode, AsciiSet, CONTROLS};

type FuncType = fn(&RcString) -> Value;

// https://url.spec.whatwg.org/#fragment-percent-encode-set
const ENCODE_FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

fn decode(str: &RcString) -> Value {
    Value::string(
        percent_decode(str.as_bytes())
            .decode_utf8()
            .unwrap()
            .borrow(),
    )
}

fn encode(str: &RcString) -> Value {
    Value::string(utf8_percent_encode(str, ENCODE_FRAGMENT).to_string())
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct URI;

impl BuiltIn for URI {
    const NAME: &'static str = "URI";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let decode_uri = FunctionBuilder::new(context, Self::decode_uri)
            .name("decodeURI")
            .length(1)
            .callable(true)
            .constructable(false)
            .build();

        let encode_uri = FunctionBuilder::new(context, Self::encode_uri)
            .name("encodeURI")
            .length(1)
            .callable(true)
            .constructable(false)
            .build();

        context.register_global_property("decodeURI", decode_uri, Attribute::all());
        context.register_global_property("encodeURI", encode_uri, Attribute::all());

        let global = context.global_object();

        (Self::NAME, Value::undefined(), Self::attribute())
    }
}

impl URI {
    pub(crate) const LENGTH: usize = 1;

    pub(crate) fn handle_uri(args: &[Value], cb: FuncType) -> Result<Value> {
        let first_arg = match args.get(0) {
            Some(Value::String(ref str)) => {
                if str.len() == 0 {
                    Value::string("")
                } else {
                    cb(str)
                }
            }
            _ => Value::Undefined,
        };

        Ok(first_arg)
    }

    pub(crate) fn decode_uri(_: &Value, args: &[Value], _context: &mut Context) -> Result<Value> {
        Self::handle_uri(args, decode)
    }

    pub(crate) fn encode_uri(_: &Value, args: &[Value], _context: &mut Context) -> Result<Value> {
        Self::handle_uri(args, encode)
    }
}
