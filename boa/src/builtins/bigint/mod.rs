//! This module implements the global `BigInt` object.
//!
//! `BigInt` is a built-in object that provides a way to represent whole numbers larger
//! than the largest number JavaScript can reliably represent with the Number primitive
//! and represented by the `Number.MAX_SAFE_INTEGER` constant.
//! `BigInt` can be used for arbitrarily large integers.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-bigint-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt

use crate::{
    builtins::{
        function::make_constructor_fn,
        value::{ResultValue, Value},
    },
    exec::Interpreter,
    syntax::ast::bigint::BigInt,
};

#[cfg(test)]
mod tests;

/// `BigInt()` function.
///
/// More Information https://tc39.es/ecma262/#sec-number-constructor-number-value
pub fn make_bigint(_this: &mut Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let data = match args.get(0) {
        Some(ref value) => {
            if let Some(bigint) = value.to_bigint() {
                Value::from(bigint)
            } else {
                panic!("RangeError: The value cannot be converted to a BigInt because it is not an integer");
            }
        }
        None => Value::from(BigInt::from(0)),
    };
    Ok(data)
}

/// `BigInt.prototype.toString( [radix] )`
///
/// The `toString()` method returns a string representing the specified BigInt object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-bigint.prototype.tostring
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/toString
pub fn to_string(this: &mut Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let radix = if !args.is_empty() {
        args[0].to_integer()
    } else {
        10
    };
    if radix < 2 && radix > 36 {
        panic!("RangeError: toString() radix argument must be between 2 and 36");
    }
    Ok(Value::from(
        this.to_bigint().unwrap().to_str_radix(radix as u32),
    ))
}

// /// `BigInt.prototype.valueOf()`
// ///
// /// The `valueOf()` method returns the wrapped primitive value of a Number object.
// ///
// /// More information:
// ///  - [ECMAScript reference][spec]
// ///  - [MDN documentation][mdn]
// ///
/// [spec]: https://tc39.es/ecma262/#sec-bigint.prototype.valueof
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/valueOf
pub fn value_of(this: &mut Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    Ok(Value::from(
        this.to_bigint().expect("BigInt.prototype.valueOf"),
    ))
}

/// Create a new `Number` object
pub fn create(global: &Value) -> Value {
    let prototype = Value::new_object(Some(global));
    prototype.set_internal_slot("BigIntData", Value::from(BigInt::from(0)));

    make_builtin_fn!(to_string, named "toString", with length 1, of prototype);
    make_builtin_fn!(value_of, named "valueOf", of prototype);

    make_constructor_fn(make_bigint, global, prototype)
}

/// Initialise the `BigInt` object on the global object.
#[inline]
pub fn init(global: &Value) {
    global.set_field_slice("BigInt", create(global));
}
