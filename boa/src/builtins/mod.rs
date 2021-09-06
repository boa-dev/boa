//! Builtins live here, such as Object, String, Math, etc.

// builtins module has a lot of built-in functions that need unnecessary_wraps
#![allow(clippy::unnecessary_wraps)]

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
pub mod reflect;
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
    reflect::Reflect,
    regexp::RegExp,
    set::set_iterator::SetIterator,
    set::Set,
    string::String,
    symbol::Symbol,
    undefined::Undefined,
};
use crate::{
    property::{Attribute, PropertyDescriptor},
    Context, JsValue,
};

pub(crate) trait BuiltIn {
    /// The binding name of the property.
    const NAME: &'static str;

    fn attribute() -> Attribute;
    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute);
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
        Reflect::init,
        #[cfg(feature = "console")]
        console::Console::init,
    ];

    let global_object = context.global_object();

    for init in &globals {
        let (name, value, attribute) = init(context);
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        global_object.borrow_mut().insert(name, property);
    }
}

pub trait JsArgs {
    /// Utility function to `get` a parameter from
    /// a `[JsValue]` or default to `JsValue::Undefined`
    /// if `get` returns `None`.
    ///
    /// Call this if you are thinking of calling something similar to
    /// `args.get(n).cloned().unwrap_or_default()` or
    /// `args.get(n).unwrap_or(&undefined)`.
    ///
    /// This returns a reference for efficiency, in case
    /// you only need to call methods of `JsValue`, so
    /// try to minimize calling `clone`.
    fn get_or_undefined(&self, index: usize) -> &JsValue;
}

impl JsArgs for [JsValue] {
    fn get_or_undefined(&self, index: usize) -> &JsValue {
        const UNDEFINED: &JsValue = &JsValue::Undefined;
        self.get(index).unwrap_or(UNDEFINED)
    }
}
