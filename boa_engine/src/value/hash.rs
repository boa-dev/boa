use super::{JsValue, JsVariant};
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
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Hash for JsValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.variant() {
            JsVariant::Undefined => UndefinedHashable.hash(state),
            JsVariant::Null => NullHashable.hash(state),
            JsVariant::String(ref string) => string.hash(state),
            JsVariant::Boolean(boolean) => boolean.hash(state),
            JsVariant::Integer(integer) => RationalHashable(f64::from(integer)).hash(state),
            JsVariant::BigInt(ref bigint) => bigint.hash(state),
            JsVariant::Rational(rational) => RationalHashable(rational).hash(state),
            JsVariant::Symbol(ref symbol) => Hash::hash(symbol, state),
            JsVariant::Object(ref object) => std::ptr::hash(object.as_ref(), state),
        }
    }
}
