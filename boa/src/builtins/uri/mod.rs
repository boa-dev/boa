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
use crate::{
    builtins::BuiltIn,
    object::{ConstructorBuilder, ObjectData, PROTOTYPE},
    property::Attribute,
    value::{AbstractRelation, IntegerOrInfinity, Value},
    BoaProfiler, Context, Result,
};
use num_traits::{float::FloatCore, Num};

const PARSE_INT_MAX_ARG_COUNT: usize = 1;

/// `Number` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EncodeURI;

fn get_utf(url: &str) -> Vec<u32> {
    let mut foo: Vec<u32> = vec![];
    for ch in url.chars() {
        foo.push(ch as u32);
    }
    foo
}

fn print_utf8(utf_str: Vec<u32>) {
    for code in utf_str {
        println!("{:04X}", code)
    }
}

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
        .static_property("AGE", 42, attribute)
        .build();

        let global = context.global_object();
        make_builtin_fn(
            Self::get_age,
            "getAge",
            &global,
            PARSE_INT_MAX_ARG_COUNT,
            context,
        );
        (Self::NAME, number_object.into(), Self::attribute())
    }
}

impl EncodeURI {
    pub(crate) const LENGTH: usize = 1;

    pub(crate) fn get_age(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        Ok(Value::Integer(42))
    }

    pub(crate) fn constructor(
        new_target: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        Ok(Value::Integer(42))
    }
}
