use super::*;

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
