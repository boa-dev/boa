//! Builtins live here, such as Object, String, Math, etc.

pub mod array;
pub mod bigint;
pub mod boolean;
#[cfg(feature = "console")]
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
pub mod set;
pub mod string;
pub mod symbol;
pub mod undefined;

pub(crate) use self::{
    array::{array_iterator::ArrayIterator, Array},
    bigint::BigInt,
    boolean::Boolean,
    date::Date,
    error::{Error, EvalError, RangeError, ReferenceError, SyntaxError, TypeError, UriError},
    function::BuiltInFunctionObject,
    global_this::GlobalThis,
    infinity::Infinity,
    json::Json,
    map::map_iterator::MapIterator,
    map::Map,
    math::Math,
    nan::NaN,
    number::Number,
    object::for_in_iterator::ForInIterator,
    object::Object as BuiltInObjectObject,
    regexp::RegExp,
    set::set_iterator::SetIterator,
    set::Set,
    string::String,
    symbol::Symbol,
    undefined::Undefined,
};
use crate::{
    property::{Attribute, DataDescriptor},
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
        Array::init,
        BigInt::init,
        Boolean::init,
        Date::init,
        Map::init,
        Number::init,
        Set::init,
        String::init,
        RegExp::init,
        Symbol::init,
        Error::init,
        RangeError::init,
        ReferenceError::init,
        TypeError::init,
        SyntaxError::init,
        EvalError::init,
        UriError::init,
        #[cfg(feature = "console")]
        console::Console::init,
    ];

    let global_object = context.global_object().clone();

    for init in &globals {
        let (name, value, attribute) = init(context);
        let property = DataDescriptor::new(value, attribute);
        global_object.borrow_mut().insert(name, property);
    }
}
