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
        new_func.set_call(func);
        let new_func_obj = to_value(new_func);
        new_func_obj.set_field_slice("length", to_value($l));
        $p.set_field_slice($name, new_func_obj);
    };
    ($fn:ident, named $name:expr, of $p:ident) => {
        make_builtin_fn!($fn, named $name, with length 0, of $p);
    };
}

/// Macro to create a new constructor function
///
/// Either (construct_body, global, prototype)
macro_rules! make_constructor_fn {
    ($body:ident, $global:ident, $proto:ident) => {{
        // Create the native function
        let constructor_fn = crate::builtins::function::Function::create_builtin(
            vec![],
            crate::builtins::function::FunctionBody::BuiltIn($body),
        );

        // Get reference to Function.prototype
        let func_prototype = $global
            .get_field_slice("Function")
            .get_field_slice(PROTOTYPE);

        // Create the function object and point its instance prototype to Function.prototype
        let mut constructor_obj = Object::function();
        constructor_obj.set_construct(constructor_fn);

        constructor_obj.set_internal_slot("__proto__", func_prototype);
        let constructor_val = to_value(constructor_obj);

        // Set proto.constructor -> constructor_obj
        $proto.set_field_slice("constructor", constructor_val.clone());
        constructor_val.set_field_slice(PROTOTYPE, $proto);

        constructor_val
    }};
    ($construct_body:ident, $call_body:ident, $global:ident, $proto:ident) => {{
        // Create the native functions
        let construct_fn = crate::builtins::function::Function::create_builtin(
            vec![],
            crate::builtins::function::FunctionBody::BuiltIn($construct_body),
        );
        let call_fn = crate::builtins::function::Function::create_builtin(
            vec![],
            crate::builtins::function::FunctionBody::BuiltIn($call_body),
        );

        // Get reference to Function.prototype
        let func_prototype = $global
            .get_field_slice("Function")
            .get_field_slice(PROTOTYPE);

        // Create the function object and point its instance prototype to Function.prototype
        let mut constructor_obj = Object::function();
        constructor_obj.set_construct(construct_fn);
        constructor_obj.set_call(call_fn);
        constructor_obj.set_internal_slot("__proto__", func_prototype);
        let constructor_val = to_value(constructor_obj);

        // Set proto.constructor -> constructor_obj
        $proto.set_field_slice("constructor", constructor_val.clone());
        constructor_val.set_field_slice(PROTOTYPE, $proto);

        constructor_val
    }};
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
