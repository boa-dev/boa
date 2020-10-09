use super::Value;

/// Possible types of values as defined at https://tc39.es/ecma262/#sec-typeof-operator.
/// Note that an object which implements call is referred to here as 'Function'.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Symbol,
    BigInt,
    Object,
    Function,
}

impl Type {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::String => "string",
            Self::Boolean => "boolean",
            Self::Symbol => "symbol",
            Self::Null => "object",
            Self::Undefined => "undefined",
            Self::Function => "function",
            Self::Object => "object",
            Self::BigInt => "bigint",
        }
    }
}

impl Value {
    /// Get the type of the value.
    ///
    /// This is similar to typeof as described at https://tc39.es/ecma262/#sec-typeof-operator but instead of
    /// returning a string it returns a Type enum which implements fmt::Display to allow getting the string if
    /// required using to_string().
    pub fn get_type(&self) -> Type {
        match *self {
            Self::Rational(_) | Self::Integer(_) => Type::Number,
            Self::String(_) => Type::String,
            Self::Boolean(_) => Type::Boolean,
            Self::Symbol(_) => Type::Symbol,
            Self::Null => Type::Null,
            Self::Undefined => Type::Undefined,
            Self::BigInt(_) => Type::BigInt,
            Self::Object(ref object) => {
                if object.is_function() {
                    Type::Function
                } else {
                    Type::Object
                }
            }
        }
    }
}
