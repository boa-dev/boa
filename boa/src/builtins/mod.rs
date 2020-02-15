/// Macro to create a new member function of a prototype
/// If no length is provided, the length will be set to 0.
macro_rules! make_builtin_fn {
    ($fn:ident, named $name:expr, with length $l:tt, of $p:ident) => {
        let $fn = to_value($fn as NativeFunctionData);
        $fn.set_field_slice("length", to_value($l));
        $p.set_field_slice($name, $fn);
    };
    ($fn:ident, named $name:expr, of $p:ident) => {
        make_builtin_fn!($fn, named $name, with length 0, of $p);
    };
}

/// The global `Array` object
pub mod array;
/// the global `Symbol` Object
pub mod symbol;
// The global `Boolean` object
pub mod boolean;
/// The global `console` object
pub mod console;
/// The global `Error` object
pub mod error;
/// The global `Function` object and function value representations
pub mod function;
/// The global `JSON` object
pub mod json;
/// The global `Math` object
pub mod math;
/// The global `Number` object
pub mod number;
/// The global `Object` object
pub mod object;
/// Property, used by `Object`
pub mod property;
/// The global 'RegExp' object
pub mod regexp;
/// The global `String` object
pub mod string;
/// Javascript values, utility methods and conversion between Javascript values and Rust values
pub mod value;
