//! Builtins live here, such as Object, String, Math, etc.

pub mod array;
pub mod bigint;
pub mod boolean;
pub mod console;
pub mod error;
pub mod function;
pub mod global_this;
pub mod infinity;
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
    error::{Error, RangeError, ReferenceError, TypeError},
    global_this::GlobalThis,
    infinity::Infinity,
    json::Json,
    math::Math,
    nan::NaN,
    number::Number,
    regexp::RegExp,
    string::String,
    symbol::Symbol,
    value::{ResultValue, Value, ValueData},
};

/// Initializes builtin objects and functions
#[inline]
pub fn init(global: &Value) {
    let globals = [
        // The `Function` global must be initialized before other types.
        function::init,
        object::init,
        Array::init,
        BigInt::init,
        Boolean::init,
        Json::init,
        Math::init,
        Number::init,
        RegExp::init,
        String::init,
        Symbol::init,
        console::init,
        // Global error types.
        Error::init,
        RangeError::init,
        ReferenceError::init,
        TypeError::init,
        // Global properties.
        NaN::init,
        Infinity::init,
        GlobalThis::init,
    ];

    match global.data() {
        ValueData::Object(ref global_object) => {
            for init in &globals {
                let (name, value) = init(global);
                global_object.borrow_mut().insert_field(name, value);
            }
        }
        _ => unreachable!("expect global object"),
    }
}
