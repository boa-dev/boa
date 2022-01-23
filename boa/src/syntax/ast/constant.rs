//! This module implements the `Const` structure, which represents the primitive values in JavaScript.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-primary-expression-literals
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Literals

use crate::{
    gc::{Finalize, Trace},
    JsBigInt,
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// Literals represent values in JavaScript.
///
/// These are fixed values **not variables** that you literally provide in your script.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-primary-expression-literals
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Literals
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
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
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-string-value
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#String_literals
    String(Sym),

    /// A floating-point number literal.
    ///
    /// The exponent part is an "`e`" or "`E`" followed by an integer, which can be signed (preceded by "`+`" or "`-`").
    /// A floating-point literal must have at least one digit, and either a decimal point or "`e`" (or "`E`").
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-number-value
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Floating-point_literals
    Num(f64),

    /// Integer types can be expressed in decimal (base 10), hexadecimal (base 16), octal (base 8) and binary (base 2).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-number-value
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Numeric_literals
    Int(i32),

    /// BigInt provides a way to represent whole numbers larger than the largest number JavaScript
    /// can reliably represent with the `Number` primitive.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-bigint-value
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Numeric_literals
    BigInt(JsBigInt),

    /// The Boolean type has two literal values: `true` and `false`.
    ///
    /// The Boolean object is a wrapper around the primitive Boolean data type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-boolean-value
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Boolean_literals
    Bool(bool),

    /// In JavaScript, `null` is marked as one of the primitive values, cause it's behaviour is seemingly primitive.
    ///
    /// In computer science, a null value represents a reference that points,
    /// generally intentionally, to a nonexistent or invalid object or address.
    /// The meaning of a null reference varies among language implementations.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-null-value
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/null
    Null,

    /// The `undefined` is a primitive value automatically assigned to variables that have just been declared, or to formal arguments for which there are no actual arguments.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-undefined
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/undefined
    Undefined,
}

impl From<Sym> for Const {
    fn from(string: Sym) -> Self {
        Self::String(string)
    }
}

impl From<f64> for Const {
    fn from(num: f64) -> Self {
        Self::Num(num)
    }
}

impl From<i32> for Const {
    fn from(i: i32) -> Self {
        Self::Int(i)
    }
}

impl From<JsBigInt> for Const {
    fn from(i: JsBigInt) -> Self {
        Self::BigInt(i)
    }
}

impl From<bool> for Const {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl ToInternedString for Const {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match *self {
            Self::String(st) => {
                format!("\"{}\"", interner.resolve(st).expect("string disappeared"))
            }
            Self::Num(num) => num.to_string(),
            Self::Int(num) => num.to_string(),
            Self::BigInt(ref num) => num.to_string(),
            Self::Bool(v) => v.to_string(),
            Self::Null => "null".to_owned(),
            Self::Undefined => "undefined".to_owned(),
        }
    }
}
