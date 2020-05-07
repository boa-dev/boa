use super::*;

impl PartialEq for ValueData {
    fn eq(&self, other: &Self) -> bool {
        match (self.clone(), other.clone()) {
            // TODO: fix this
            // _ if self.ptr.to_inner() == &other.ptr.to_inner() => true,
            _ if self.is_null_or_undefined() && other.is_null_or_undefined() => true,
            (Self::String(_), _) | (_, Self::String(_)) => self.to_string() == other.to_string(),
            (Self::Boolean(a), Self::Boolean(b)) if a == b => true,
            (Self::Rational(a), Self::Rational(b)) if a == b && !a.is_nan() && !b.is_nan() => true,
            (Self::Rational(a), _) if a == other.to_number() => true,
            (_, Self::Rational(a)) if a == self.to_number() => true,
            (Self::Integer(a), Self::Integer(b)) if a == b => true,
            _ => false,
        }
    }
}

impl Add for ValueData {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::String(ref s), ref o) => {
                Self::String(format!("{}{}", s.clone(), &o.to_string()))
            }
            (ref s, Self::String(ref o)) => Self::String(format!("{}{}", s.to_string(), o)),
            (ref s, ref o) => Self::Rational(s.to_number() + o.to_number()),
        }
    }
}
impl Sub for ValueData {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::Rational(self.to_number() - other.to_number())
    }
}
impl Mul for ValueData {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::Rational(self.to_number() * other.to_number())
    }
}
impl Div for ValueData {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self::Rational(self.to_number() / other.to_number())
    }
}
impl Rem for ValueData {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        Self::Rational(self.to_number() % other.to_number())
    }
}
impl BitAnd for ValueData {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self::Integer(self.to_integer() & other.to_integer())
    }
}
impl BitOr for ValueData {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self::Integer(self.to_integer() | other.to_integer())
    }
}
impl BitXor for ValueData {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        Self::Integer(self.to_integer() ^ other.to_integer())
    }
}
impl Shl for ValueData {
    type Output = Self;
    fn shl(self, other: Self) -> Self {
        Self::Integer(self.to_integer() << other.to_integer())
    }
}
impl Shr for ValueData {
    type Output = Self;
    fn shr(self, other: Self) -> Self {
        Self::Integer(self.to_integer() >> other.to_integer())
    }
}
impl Not for ValueData {
    type Output = Self;
    fn not(self) -> Self {
        Self::Boolean(!self.is_true())
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
        let x_ptr = Gc::into_raw(x.clone());
        let y_ptr = Gc::into_raw(y.clone());
        return x_ptr == y_ptr;
    }

    if x.get_type() != y.get_type() {
        return false;
    }

    if x.get_type() == "number" {
        let native_x: f64 = from_value(x.clone()).expect("failed to get value");
        let native_y: f64 = from_value(y.clone()).expect("failed to get value");
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
        "boolean" => {
            from_value::<bool>(x.clone()).expect("failed to get value")
                == from_value::<bool>(y.clone()).expect("failed to get value")
        }
        "object" => *x == *y,
        _ => false,
    }
}
