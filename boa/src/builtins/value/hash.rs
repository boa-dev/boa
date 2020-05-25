use super::*;

use crate::builtins::Number;
use std::hash::{Hash, Hasher};

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        same_value_zero(self, other)
    }
}

impl Eq for Value {}

#[derive(PartialEq, Eq, Hash)]
struct UndefinedHashable;

#[derive(PartialEq, Eq, Hash)]
struct NullHashable;

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

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let data = self.data();
        match data {
            ValueData::Undefined => UndefinedHashable.hash(state),
            ValueData::Null => NullHashable.hash(state),
            ValueData::String(ref string) => string.hash(state),
            ValueData::Boolean(boolean) => boolean.hash(state),
            ValueData::Integer(integer) => integer.hash(state),
            ValueData::BigInt(ref bigint) => bigint.hash(state),
            ValueData::Rational(rational) => RationalHashable(*rational).hash(state),
            ValueData::Symbol(_) | ValueData::Object(_) => std::ptr::hash(data, state),
        }
    }
}
