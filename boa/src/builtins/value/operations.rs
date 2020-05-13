use super::*;
use crate::builtins::number;
use crate::Interpreter;

use std::borrow::Borrow;

impl Value {
    /// Strict equality comparison.
    ///
    /// This method is executed when doing strict equality comparisons with the `===` operator.
    /// For more information, check <https://tc39.es/ecma262/#sec-strict-equality-comparison>.
    pub fn strict_equals(&self, other: &Self) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        if self.is_number() {
            return number::equals(self, other);
        }

        same_value_non_number(self, other)
    }

    /// Abstract equality comparison.
    ///
    /// This method is executed when doing abstract equality comparisons with the `==` operator.
    ///  For more information, check <https://tc39.es/ecma262/#sec-abstract-equality-comparison>
    pub fn equals(&mut self, other: &mut Self, interpreter: &mut Interpreter) -> bool {
        if self.get_type() == other.get_type() {
            return self.strict_equals(other);
        }

        match (self.data(), other.data()) {
            _ if self.is_null_or_undefined() && other.is_null_or_undefined() => true,

            // https://github.com/rust-lang/rust/issues/54883
            (ValueData::Integer(_), ValueData::String(_))
            | (ValueData::Rational(_), ValueData::String(_))
            | (ValueData::String(_), ValueData::Integer(_))
            | (ValueData::String(_), ValueData::Rational(_))
            | (ValueData::Rational(_), ValueData::Boolean(_))
            | (ValueData::Integer(_), ValueData::Boolean(_)) => {
                let a: &Value = self.borrow();
                let b: &Value = other.borrow();
                number::equals(a, b)
            }
            (ValueData::Boolean(_), _) => {
                other.equals(&mut Value::from(self.to_integer()), interpreter)
            }
            (_, ValueData::Boolean(_)) => {
                self.equals(&mut Value::from(other.to_integer()), interpreter)
            }
            (ValueData::Object(_), _) => {
                let mut primitive = interpreter.to_primitive(self, None);
                primitive.equals(other, interpreter)
            }
            (_, ValueData::Object(_)) => {
                let mut primitive = interpreter.to_primitive(other, None);
                primitive.equals(self, interpreter)
            }
            _ => false,
        }
    }
}

impl Add for Value {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::String(ref s), ref o) => {
                Self::string(format!("{}{}", s.clone(), &o.to_string()))
            }
            (ref s, ValueData::String(ref o)) => Self::string(format!("{}{}", s.to_string(), o)),
            (ref s, ref o) => Self::rational(s.to_number() + o.to_number()),
        }
    }
}
impl Sub for Value {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::rational(self.to_number() - other.to_number())
    }
}
impl Mul for Value {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::rational(self.to_number() * other.to_number())
    }
}
impl Div for Value {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self::rational(self.to_number() / other.to_number())
    }
}
impl Rem for Value {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        Self::rational(self.to_number() % other.to_number())
    }
}
impl BitAnd for Value {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self::integer(self.to_integer() & other.to_integer())
    }
}
impl BitOr for Value {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self::integer(self.to_integer() | other.to_integer())
    }
}
impl BitXor for Value {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        Self::integer(self.to_integer() ^ other.to_integer())
    }
}
impl Shl for Value {
    type Output = Self;
    fn shl(self, other: Self) -> Self {
        Self::integer(self.to_integer() << other.to_integer())
    }
}
impl Shr for Value {
    type Output = Self;
    fn shr(self, other: Self) -> Self {
        Self::integer(self.to_integer() >> other.to_integer())
    }
}
impl Not for Value {
    type Output = Self;
    fn not(self) -> Self {
        Self::boolean(!self.is_true())
    }
}

/// The internal comparison abstract operation SameValueZero(x, y),
/// where x and y are ECMAScript language values, produces true or false.
/// SameValueZero differs from SameValue only in its treatment of +0 and -0.
///
/// Such a comparison is performed as follows:
///
/// https://tc39.es/ecma262/#sec-samevaluezero
pub fn same_value_zero(x: &Value, y: &Value) -> bool {
    if x.get_type() != y.get_type() {
        return false;
    }

    if x.is_number() {
        return number::same_value_zero(x, y);
    }

    same_value_non_number(x, y)
}

/// The internal comparison abstract operation SameValue(x, y),
/// where x and y are ECMAScript language values, produces true or false.
/// Such a comparison is performed as follows:
///
/// https://tc39.es/ecma262/#sec-samevalue
/// strict mode currently compares the pointers
pub fn same_value(x: &Value, y: &Value, strict: bool) -> bool {
    if strict {
        // Do both Values point to the same underlying valueData?
        let x_ptr = Gc::into_raw(x.0.clone());
        let y_ptr = Gc::into_raw(y.0.clone());
        return x_ptr == y_ptr;
    }

    if x.get_type() != y.get_type() {
        return false;
    }

    if x.is_number() {
        return number::same_value(x, y);
    }

    same_value_non_number(x, y)
}

pub fn same_value_non_number(x: &Value, y: &Value) -> bool {
    debug_assert!(x.get_type() == y.get_type());
    match x.get_type() {
        "undefined" => true,
        "null" => true,
        "string" => {
            if x.to_string() == y.to_string() {
                return true;
            }
            false
        }
        "boolean" => bool::from(x) == bool::from(y),
        "object" => std::ptr::eq(x, y),
        _ => false,
    }
}
