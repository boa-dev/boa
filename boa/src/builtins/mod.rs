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
    function::Function,
    number::Number,
    regexp::RegExp,
    string::String,
    value::{ResultValue, Value},
};

/// Initializes builtin objects and functions
#[inline]
pub fn init(global: &Value) {
    Array::init(global);
    BigInt::init(global);
    Boolean::init(global);
    global_this::init(global);
    json::init(global);
    math::init(global);
    nan::init(global);
    Number::init(global);
    object::init(global);
    function::init(global);
    RegExp::init(global);
    String::init(global);
    symbol::init(global);
    console::init(global);
    Error::init(global);
    RangeError::init(global);
    TypeError::init(global);
}
