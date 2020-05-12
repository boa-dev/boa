use super::*;
use crate::Interpreter;
use std::borrow::Borrow;

#[allow(clippy::float_cmp)]
/// https://tc39.es/ecma262/#sec-numeric-types-number-equal
fn strict_number_equals<T: Into<f64>>(a: T, b: T) -> bool {
    let a: f64 = a.into();
    let b: f64 = b.into();

    if a.is_nan() || b.is_nan() {
        return false;
    }

    a == b
}

impl Value {
    // https://tc39.es/ecma262/#sec-strict-equality-comparison
    pub fn strict_equals(&self, other: &Self) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        if self.is_number() {
            return strict_number_equals(self, other)
        }

        same_value_non_number(self, other)
    }

    // https://tc39.es/ecma262/#sec-abstract-equality-comparison
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
                strict_number_equals(a, b)
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self.data(), other.data()) {
            // TODO: fix this
            // _ if self.ptr.to_inner() == &other.ptr.to_inner() => true,
            _ if self.is_null_or_undefined() && other.is_null_or_undefined() => true,
            (ValueData::String(_), _) | (_, ValueData::String(_)) => {
                self.to_string() == other.to_string()
            }
            (ValueData::Boolean(a), ValueData::Boolean(b)) if a == b => true,
            (ValueData::Rational(a), ValueData::Rational(b))
                if a == b && !a.is_nan() && !b.is_nan() =>
            {
                true
            }
            (ValueData::Rational(a), _) if *a == other.to_number() => true,
            (_, ValueData::Rational(a)) if *a == self.to_number() => true,
            (ValueData::Integer(a), ValueData::Integer(b)) if a == b => true,
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

    if x.get_type() == "number" {
        let native_x = f64::from(x);
        let native_y = f64::from(y);
        return native_x.abs() - native_y.abs() == 0.0;
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
        "object" => *x == *y,
        _ => false,
    }
}

// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Equality_comparisons_and_sameness
#[test]
fn abstract_equality_comparison() {
    use crate::{forward, Executor, Realm};
    let realm = Realm::create();
    let mut engine = Executor::new(realm);

    assert_eq!(forward(&mut engine, "undefined == undefined"), "true");
    assert_eq!(forward(&mut engine, "null == null"), "true");
    assert_eq!(forward(&mut engine, "true == true"), "true");
    assert_eq!(forward(&mut engine, "false == false"), "true");
    assert_eq!(forward(&mut engine, "'foo' == 'foo'"), "true");
    assert_eq!(forward(&mut engine, "0 == 0"), "true");
    assert_eq!(forward(&mut engine, "+0 == -0"), "true");
    assert_eq!(forward(&mut engine, "+0 == 0"), "true");
    assert_eq!(forward(&mut engine, "-0 == 0"), "true");
    assert_eq!(forward(&mut engine, "0 == false"), "true");
    assert_eq!(forward(&mut engine, "'' == false"), "true");
    assert_eq!(forward(&mut engine, "'' == 0"), "true");
    assert_eq!(forward(&mut engine, "'17' == 17"), "true");
    assert_eq!(forward(&mut engine, "[1,2] == '1,2'"), "true");
    assert_eq!(forward(&mut engine, "new String('foo') == 'foo'"), "true");
    assert_eq!(forward(&mut engine, "null == undefined"), "true");
    assert_eq!(forward(&mut engine, "undefined == null"), "true");
    assert_eq!(forward(&mut engine, "null == false"), "false");
    assert_eq!(
        forward(&mut engine, "a = { foo: 'bar' }; b = { foo: 'bar'}; a == b"),
        "false"
    );
    assert_eq!(
        forward(&mut engine, "new String('foo') == new String('foo')"),
        "false"
    );
    assert_eq!(forward(&mut engine, "0 == null"), "false");

    // https://github.com/jasonwilliams/boa/issues/357
    assert_eq!(forward(&mut engine, "0 == '-0'"), "true");
    assert_eq!(forward(&mut engine, "0 == '+0'"), "true");
    assert_eq!(forward(&mut engine, "'+0' == 0"), "true");
    assert_eq!(forward(&mut engine, "'-0' == 0"), "true");

    // https://github.com/jasonwilliams/boa/issues/393
    // assert_eq!(forward(&mut engine, "0 == NaN"), "false");
    // assert_eq!(forward(&mut engine, "'foo' == NaN"), "false");
    // assert_eq!(forward(&mut engine, "NaN == NaN"), "false");
}
