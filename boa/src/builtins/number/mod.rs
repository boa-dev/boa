//! This module implements the global `Number` object.
//!
//! The `Number` JavaScript object is a wrapper object allowing you to work with numerical values.
//! A `Number` object is created using the `Number()` constructor. A primitive type object number is created using the `Number()` **function**.
//!
//! The JavaScript `Number` type is double-precision 64-bit binary format IEEE 754 value. In more recent implementations,
//! JavaScript also supports integers with arbitrary precision using the BigInt type.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-number-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number

#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        function::NativeFunctionData,
        object::{internal_methods_trait::ObjectInternalMethods, Object, ObjectKind, PROTOTYPE},
        value::{to_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use std::{borrow::Borrow, f64, ops::Deref};

/// Helper function that converts a Value to a Number.
fn to_number(value: &Value) -> Value {
    match *value.deref().borrow() {
        ValueData::Boolean(b) => {
            if b {
                to_value(1)
            } else {
                to_value(0)
            }
        }
        ValueData::Function(_) | ValueData::Symbol(_) | ValueData::Undefined => to_value(f64::NAN),
        ValueData::Integer(i) => to_value(f64::from(i)),
        ValueData::Object(ref o) => (o).deref().borrow().get_internal_slot("NumberData"),
        ValueData::Null => to_value(0),
        ValueData::Rational(n) => to_value(n),
        ValueData::String(ref s) => match s.parse::<f64>() {
            Ok(n) => to_value(n),
            Err(_) => to_value(f64::NAN),
        },
    }
}

/// Helper function that formats a float as a ES6-style exponential number string.
fn num_to_exponential(n: f64) -> String {
    match n.abs() {
        x if x > 1.0 => format!("{:e}", n).replace("e", "e+"),
        x if x == 0.0 => format!("{:e}", n).replace("e", "e+"),
        _ => format!("{:e}", n),
    }
}

/// Create a new number `[[Construct]]`
pub fn make_number(this: &Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let data = match args.get(0) {
        Some(ref value) => to_number(value),
        None => to_number(&to_value(0)),
    };
    this.set_internal_slot("NumberData", data);
    Ok(this.clone())
}

/// `Number()` function.
///
/// More Information https://tc39.es/ecma262/#sec-number-constructor-number-value
pub fn call_number(_this: &Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let data = match args.get(0) {
        Some(ref value) => to_number(value),
        None => to_number(&to_value(0)),
    };
    Ok(data)
}

/// `Number.prototype.toExponential( [fractionDigits] )`
///
/// The `toExponential()` method returns a string representing the Number object in exponential notation.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.toexponential
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toExponential
pub fn to_exponential(this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this).to_num();
    let this_str_num = num_to_exponential(this_num);
    Ok(to_value(this_str_num))
}

/// `Number.prototype.toFixed( [digits] )`
///
/// The `toFixed()` method formats a number using fixed-point notation
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.tofixed
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toFixed
pub fn to_fixed(this: &Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this).to_num();
    let precision = match args.get(0) {
        Some(n) => match n.to_int() {
            x if x > 0 => n.to_int() as usize,
            _ => 0,
        },
        None => 0,
    };
    let this_fixed_num = format!("{:.*}", precision, this_num);
    Ok(to_value(this_fixed_num))
}

/// `Number.prototype.toLocaleString( [locales [, options]] )`
///
/// The `toLocaleString()` method returns a string with a language-sensitive representation of this number.
///
/// Note that while this technically conforms to the Ecma standard, it does no actual
/// internationalization logic.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.tolocalestring
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toLocaleString
pub fn to_locale_string(this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this).to_num();
    let this_str_num = format!("{}", this_num);
    Ok(to_value(this_str_num))
}

/// `Number.prototype.toPrecision( [precision] )`
///
/// The `toPrecision()` method returns a string representing the Number object to the specified precision.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.toexponential
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toPrecision
pub fn to_precision(this: &Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this);
    let _num_str_len = format!("{}", this_num.to_num()).len();
    let _precision = match args.get(0) {
        Some(n) => match n.to_int() {
            x if x > 0 => n.to_int() as usize,
            _ => 0,
        },
        None => 0,
    };
    // TODO: Implement toPrecision
    unimplemented!("TODO: Implement toPrecision");
}

/// `Number.prototype.toString( [radix] )`
///
/// The `toString()` method returns a string representing the specified Number object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.tostring
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toString
pub fn to_string(this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    Ok(to_value(format!("{}", to_number(this).to_num())))
}

/// `Number.prototype.toString()`
///
/// The `valueOf()` method returns the wrapped primitive value of a Number object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.valueof
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/valueOf
pub fn value_of(this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    Ok(to_number(this))
}

/// Create a new `Number` object
pub fn create_constructor(global: &Value) -> Value {
    let mut number_constructor = Object::default();
    number_constructor.kind = ObjectKind::Function;

    number_constructor.set_internal_method("construct", make_number);
    number_constructor.set_internal_method("call", call_number);

    let number_prototype = ValueData::new_obj(Some(global));

    number_prototype.set_internal_slot("NumberData", to_value(0));

    make_builtin_fn!(to_exponential, named "toExponential", with length 1, of number_prototype);
    make_builtin_fn!(to_fixed, named "toFixed", with length 1, of number_prototype);
    make_builtin_fn!(to_locale_string, named "toLocaleString", of number_prototype);
    make_builtin_fn!(to_precision, named "toPrecision", with length 1, of number_prototype);
    make_builtin_fn!(to_string, named "toString", with length 1, of number_prototype);
    make_builtin_fn!(value_of, named "valueOf", of number_prototype);

    let number = to_value(number_constructor);
    number_prototype.set_field_slice("constructor", number.clone());
    number.set_field_slice(PROTOTYPE, number_prototype);
    number
}
