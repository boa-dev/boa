use super::Value;

/// Possible types of values as defined at <https://tc39.es/ecma262/#sec-typeof-operator>.
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
}

impl Value {
    /// Get the type of a value
    ///
    /// This is the abstract operation Type(v), as described in
    /// <https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types>
    /// so it treats `Type::Function` objects and `Type::Object` objects as `Type::Object`.
    /// If you instead need to call the `typeof` operator, check [`Value::type_of`]
    pub fn get_type(&self) -> Type {
        match *self {
            Self::Rational(_) | Self::Integer(_) => Type::Number,
            Self::String(_) => Type::String,
            Self::Boolean(_) => Type::Boolean,
            Self::Symbol(_) => Type::Symbol,
            Self::Null => Type::Null,
            Self::Undefined => Type::Undefined,
            Self::BigInt(_) => Type::BigInt,
            Self::Object(_) => Type::Object,
        }
    }
}
