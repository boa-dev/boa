//! This module implements the `Const` structure, which represents the primitive values in JavaScript.

use gc_derive::{Finalize, Trace};
use std::fmt::{Display, Formatter, Result};

#[cfg(feature = "serde-ast")]
use serde::{Deserialize, Serialize};

/// Literals represent values in JavaScript.
///
/// These are fixed values **not variables** that you literally provide in your script.
///
/// More information:
///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-primary-expression-literals)
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Literals
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum Const {
    /// A string literal is zero or more characters enclosed in double (`"`) or single (`'`) quotation marks.
    ///
    /// A string must be delimited by quotation marks of the same type (that is, either both single quotation marks, or both double quotation marks).
    /// You can call any of the String object's methods on a string literal value.
    /// JavaScript automatically converts the string literal to a temporary String object,
    /// calls the method, then discards the temporary String object.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-terms-and-definitions-string-value)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#String_literals
    String(String),

    /// A floating-point number literal.
    ///
    /// The exponent part is an "`e`" or "`E`" followed by an integer, which can be signed (preceded by "`+`" or "`-`").
    /// A floating-point literal must have at least one digit, and either a decimal point or "`e`" (or "`E`").
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-terms-and-definitions-number-value)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Floating-point_literals
    Num(f64),

    /// Integer types can be expressed in decimal (base 10), hexadecimal (base 16), octal (base 8) and binary (base 2).
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-terms-and-definitions-number-value)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Numeric_literals
    Int(i32),

    /// The Boolean type has two literal values: `true` and `false`.
    ///
    /// The Boolean object is a wrapper around the primitive Boolean data type.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-terms-and-definitions-boolean-value)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Boolean_literals
    Bool(bool),

    /// In JavaScript, `null` is marked as one of the primitive values, cause it's behaviour is seemingly primitive.
    ///
    /// In computer science, a null value represents a reference that points,
    /// generally intentionally, to a nonexistent or invalid object or address.
    /// The meaning of a null reference varies among language implementations.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-null-value)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/null
    Null,

    /// The `undefined` is a primitive value automatically assigned to variables that have just been declared, or to formal arguments for which there are no actual arguments.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-undefined)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/undefined
    Undefined,
}

impl Display for Const {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
            Const::String(ref st) => write!(f, "\"{}\"", st),
            Const::Num(num) => write!(f, "{}", num),
            Const::Int(num) => write!(f, "{}", num),
            Const::Bool(v) => write!(f, "{}", v),
            Const::Null => write!(f, "null"),
            Const::Undefined => write!(f, "undefined"),
        }
    }
}
