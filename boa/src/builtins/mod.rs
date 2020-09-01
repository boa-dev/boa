//! Builtins live here, such as Object, String, Math, etc.

pub mod array;
pub mod bigint;
pub mod boolean;
pub mod console;
pub mod date;
pub mod error;
pub mod function;
pub mod global_this;
pub mod infinity;
pub mod json;
pub mod map;
pub mod math;
pub mod nan;
pub mod number;
pub mod object;
pub mod property;
pub mod regexp;
pub mod string;
pub mod symbol;
pub mod undefined;

pub(crate) use self::{
    array::Array,
    bigint::BigInt,
    boolean::Boolean,
    console::Console,
    date::Date,
    error::{Error, RangeError, ReferenceError, SyntaxError, TypeError},
    global_this::GlobalThis,
    infinity::Infinity,
    json::Json,
    map::Map,
    math::Math,
    nan::NaN,
    number::Number,
    regexp::RegExp,
    string::String,
    symbol::Symbol,
    undefined::Undefined,
};
use crate::{Interpreter, Value};

/// Initializes builtin objects and functions
#[inline]
pub fn init(interpreter: &mut Interpreter) {
    let globals = [
        // The `Function` global must be initialized before other types.
        function::init,
        object::init,
        Array::init,
        BigInt::init,
        Boolean::init,
        Date::init,
        Json::init,
        Map::init,
        Math::init,
        Number::init,
        RegExp::init,
        String::init,
        Symbol::init,
        Console::init,
        // Global error types.
        Error::init,
        RangeError::init,
        ReferenceError::init,
        TypeError::init,
        SyntaxError::init,
        // Global properties.
        NaN::init,
        Infinity::init,
        GlobalThis::init,
        Undefined::init,
    ];

    for init in &globals {
        let (name, value) = init(interpreter);
        let global = interpreter.global();
        match global {
            Value::Object(ref global_object) => {
                global_object.borrow_mut().insert_field(name, value);
            }
            _ => unreachable!("expect global object"),
        }
    }
}
