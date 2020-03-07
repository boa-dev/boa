use gc_derive::{Finalize, Trace};
use std::fmt::{Display, Formatter, Result};

/// A Javascript Constant
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum Const {
    /// A UTF-8 string, such as `"Hello, world"`
    String(String),
    // A 64-bit floating-point number, such as `3.1415`
    Num(f64),
    // A 32-bit integer, such as `42`
    Int(i32),
    // A boolean, which is either `true` or `false` and is used to check if criteria are met
    Bool(bool),
    // The `null` value, which represents a non-existant value
    Null,
    // The `undefined` value, which represents a field or index that doesn't exist
    Undefined,
}

impl Display for Const {
    fn fmt(&self, f: &mut Formatter) -> Result {
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
