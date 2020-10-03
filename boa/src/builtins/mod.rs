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
pub mod iterable;
pub mod json;
pub mod map;
pub mod math;
pub mod nan;
pub mod number;
pub mod object;
pub mod regexp;
pub mod string;
pub mod symbol;
pub mod undefined;

pub(crate) use self::{
    array::{array_iterator::ArrayIterator, Array},
    bigint::BigInt,
    boolean::Boolean,
    console::Console,
    date::Date,
    error::{Error, RangeError, ReferenceError, SyntaxError, TypeError},
    function::BuiltInFunctionObject,
    global_this::GlobalThis,
    infinity::Infinity,
    json::Json,
    map::Map,
    math::Math,
    nan::NaN,
    number::Number,
    object::Object as BuiltInObjectObject,
    regexp::RegExp,
    string::String,
    symbol::Symbol,
    undefined::Undefined,
};
use crate::{
    property::{Attribute, Property},
    Context, Value,
};

pub(crate) trait BuiltIn {
    /// The binding name of the property.
    const NAME: &'static str;

    fn attribute() -> Attribute;
    fn init(context: &mut Context) -> (&'static str, Value, Attribute);
}

/// Initializes builtin objects and functions
#[inline]
pub fn init(context: &mut Context) {
    let globals = [
        // Global properties.
        Undefined::init,
        Infinity::init,
        NaN::init,
        GlobalThis::init,
        BuiltInFunctionObject::init,
        BuiltInObjectObject::init,
        Math::init,
        Json::init,
        Console::init,
        Array::init,
        BigInt::init,
        Boolean::init,
        Date::init,
        Map::init,
        Number::init,
        String::init,
        RegExp::init,
        Symbol::init,
        Error::init,
        RangeError::init,
        ReferenceError::init,
        TypeError::init,
        SyntaxError::init,
    ];

    let global_object = if let Value::Object(global) = context.global_object() {
        global.clone()
    } else {
        unreachable!("global object should always be an object")
    };

    for init in &globals {
        let (name, value, attribute) = init(context);
        let property = Property::data_descriptor(value, attribute);
        global_object.borrow_mut().insert(name, property);
    }
}
