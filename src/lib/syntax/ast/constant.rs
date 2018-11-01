use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
/// A Javascript Constant
pub enum Const {
    /// A UTF-8 string, such as `"Hello, world"`
    String(String),
    // A regular expression, such as `/where('s| is) [wW]ally/`
    RegExp(String, bool, bool),
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
        return match *self {
            Const::String(ref st) => write!(f, "\"{}\"", st),
            Const::RegExp(ref reg, _, _) => write!(f, "~/{}/", reg),
            Const::Num(num) => write!(f, "{}", num),
            Const::Int(num) => write!(f, "{}", num),
            Const::Bool(v) => write!(f, "{}", v),
            Const::Null => write!(f, "null"),
            Const::Undefined => write!(f, "undefined"),
        };
    }
}
