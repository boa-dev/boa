use std::fmt;

/// Possible types of val as defined at https://tc39.es/ecma262/#sec-typeof-operator.
/// Note that an object which implements call is referred to here as 'Function'.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Type {
    Undefined,
    Null,
    Boolean,
    Number,
    Str,
    Symbol,
    BigInt,
    Object,
    Function,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number => write!(f, "number"),
            Self::Str => write!(f, "string"),
            Self::Boolean => write!(f, "boolean"),
            Self::Symbol => write!(f, "symbol"),
            Self::Null => write!(f, "object"),
            Self::Undefined => write!(f, "undefined"),
            Self::Function => write!(f, "function"),
            Self::Object => write!(f, "object"),
            Self::BigInt => write!(f, "bigint"),
        }
    }
}
