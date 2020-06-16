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
<<<<<<< HEAD
    error::{Error, RangeError, ReferenceError, TypeError},
    function::Function,
=======
    error::{Error, RangeError, TypeError},
    global_this::GlobalThis,
    infinity::Infinity,
    json::Json,
    math::Math,
    nan::NaN,
>>>>>>> master
    number::Number,
    regexp::RegExp,
    string::String,
    symbol::Symbol,
    value::{ResultValue, Value},
};

/// Initializes builtin objects and functions
#[inline]
pub fn init(global: &Value) {
<<<<<<< HEAD
    Array::init(global);
    BigInt::init(global);
    Boolean::init(global);
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
    ReferenceError::init(global);
    TypeError::init(global);
=======
    let globals = vec![
        // The `Function` global must be initialized before other types.
        function::init(global),
        Array::init(global),
        BigInt::init(global),
        Boolean::init(global),
        Json::init(global),
        Math::init(global),
        Number::init(global),
        object::init(global),
        RegExp::init(global),
        String::init(global),
        Symbol::init(global),
        console::init(global),
        // Global error types.
        Error::init(global),
        RangeError::init(global),
        ReferenceError::init(global);
        TypeError::init(global),
        // Global properties.
        NaN::init(global),
        Infinity::init(global),
        GlobalThis::init(global),
    ];

    let mut global_object = global.as_object_mut().expect("global object");
    for (name, value) in globals {
        global_object.insert_field(name, value);
    }
>>>>>>> master
}
