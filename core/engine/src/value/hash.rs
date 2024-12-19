use super::{InnerValue, JsValue};
use crate::builtins::Number;
use std::hash::{Hash, Hasher};

impl PartialEq for JsValue {
    fn eq(&self, other: &Self) -> bool {
        Self::same_value_zero(self, other)
    }
}

impl Eq for JsValue {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UndefinedHashable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NullHashable;

#[derive(Debug, Clone, Copy)]
struct RationalHashable(f64);

impl PartialEq for RationalHashable {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Number::same_value(self.0, other.0)
    }
}

impl Eq for RationalHashable {}

impl Hash for RationalHashable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Hash for JsValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.inner {
            InnerValue::Undefined => UndefinedHashable.hash(state),
            InnerValue::Null => NullHashable.hash(state),
            InnerValue::String(ref string) => string.hash(state),
            InnerValue::Boolean(boolean) => boolean.hash(state),
            InnerValue::Integer32(integer) => RationalHashable(f64::from(integer)).hash(state),
            InnerValue::BigInt(ref bigint) => bigint.hash(state),
            InnerValue::Float64(rational) => RationalHashable(rational).hash(state),
            InnerValue::Symbol(ref symbol) => Hash::hash(symbol, state),
            InnerValue::Object(ref object) => object.hash(state),
        }
    }
}
