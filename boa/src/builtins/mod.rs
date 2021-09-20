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
    object::JsObject,
    property::{Attribute, PropertyDescriptor},
    Context, JsValue,
};

pub(crate) trait BuiltIn {
    /// The binding name of the property.
    const NAME: &'static str;

    const ATTRIBUTE: Attribute;
    fn init(context: &mut Context) -> JsValue;
}

#[inline]
fn init_builtin<B: BuiltIn>(global: &JsObject, context: &mut Context) {
    let value = B::init(context);
    let property = PropertyDescriptor::builder()
        .value(value)
        .writable(B::ATTRIBUTE.writable())
        .enumerable(B::ATTRIBUTE.enumerable())
        .configurable(B::ATTRIBUTE.configurable());
    global.borrow_mut().insert(B::NAME, property);
}

/// Initializes builtin objects and functions
#[inline]
pub fn init(context: &mut Context) {
    let global_object = context.global_object();

    macro_rules! globals {
        ($( $builtin:ty ),*) => {
            $(init_builtin::<$builtin>(&global_object, context)
            );*
        }
    }

    globals! {
        Undefined,
        Infinity,
        NaN,
        GlobalThis,
        BuiltInFunctionObject,
        BuiltInObjectObject,
        Math,
        Json,
        Array,
        BigInt,
        Boolean,
        Date,
        Map,
        Number,
        Set,
        String,
        RegExp,
        Symbol,
        Error,
        RangeError,
        ReferenceError,
        TypeError,
        SyntaxError,
        EvalError,
        UriError,
        Reflect
    };

    #[cfg(feature = "console")]
    init_builtin::<console::Console>(&global_object, context);
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
