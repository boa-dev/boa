//! Builtins live here, such as Object, String, Math etc

/// Macro to create a new member function of a prototype.
///
/// If no length is provided, the length will be set to 0.
macro_rules! make_builtin_fn {
    ($fn:ident, named $name:expr, with length $l:tt, of $p:ident) => {
        let func = crate::builtins::function::Function::create_builtin(
            vec![],
            crate::builtins::function::FunctionBody::BuiltIn($fn),
        );

        let mut new_func = crate::builtins::object::Object::function();
        new_func.set_func(func);
        let new_func_obj = Value::from(new_func);
        new_func_obj.set_field_slice("length", Value::from($l));
        $p.set_field_slice($name, new_func_obj);
    };
    ($fn:ident, named $name:expr, of $p:ident) => {
        make_builtin_fn!($fn, named $name, with length 0, of $p);
    };
}

pub mod array;
pub mod boolean;
pub mod console;
pub mod error;
pub mod function;
pub mod json;
pub mod math;
pub mod number;
pub mod object;
pub mod property;
pub mod regexp;
pub mod string;
pub mod symbol;
pub mod value;

use value::Value;

/// Initializes builtin objects and functions
#[inline]
pub fn init(global: &Value) {
    array::init(global);
    boolean::init(global);
    json::init(global);
    math::init(global);
    number::init(global);
    object::init(global);
    function::init(global);
    regexp::init(global);
    string::init(global);
    symbol::init(global);
    console::init(global);
}
