//! Builtins live here, such as Object, String, Math, etc.

pub mod array;
pub mod bigint;
pub mod boolean;
pub mod console;
pub mod error;
pub mod function;
pub mod global_this;
pub mod json;
pub mod math;
pub mod nan;
pub mod number;
pub mod object;
pub mod property;
pub mod regexp;
pub mod string;
pub mod symbol;
pub mod value;

pub(crate) use self::{
    array::Array,
    bigint::BigInt,
    boolean::Boolean,
    error::{Error, RangeError, TypeError},
    global_this::GlobalThis,
    nan::NaN,
    number::Number,
    regexp::RegExp,
    string::String,
    symbol::Symbol,
    value::{ResultValue, Value},
};

/// Initializes builtin objects and functions
#[inline]
pub fn init(global: &Value) {
    let globals = vec![
        function::init(global),
        Array::init(global),
        BigInt::init(global),
        Boolean::init(global),
        json::init(global),
        math::init(global),
        Number::init(global),
        NaN::init(global),
        GlobalThis::init(global),
        object::init(global),
        RegExp::init(global),
        String::init(global),
        Symbol::init(global),
        console::init(global),
        Error::init(global),
        RangeError::init(global),
        TypeError::init(global),
    ];

    let mut global_object = global.as_object_mut().expect("global object");
    for (name, value) in globals {
        global_object.insert_field(name, value);
    }
}
