use super::*;

impl Add for Value {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::String(ref s), ref o) => {
                Self::string(format!("{}{}", s.clone(), &o.to_string()))
            }
            (ValueData::BigInt(ref n1), ValueData::BigInt(ref n2)) => {
                Self::bigint(n1.clone() + n2.clone())
            }
            (ref s, ValueData::String(ref o)) => Self::string(format!("{}{}", s.to_string(), o)),
            (ref s, ref o) => Self::rational(s.to_number() + o.to_number()),
        }
    }
}
impl Sub for Value {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::BigInt(ref a), ValueData::BigInt(ref b)) => {
                Self::bigint(a.clone() - b.clone())
            }
            (a, b) => Self::rational(a.to_number() - b.to_number()),
        }
    }
}
impl Mul for Value {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::BigInt(ref a), ValueData::BigInt(ref b)) => {
                Self::bigint(a.clone() * b.clone())
            }
            (a, b) => Self::rational(a.to_number() * b.to_number()),
        }
    }
}
impl Div for Value {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::BigInt(ref a), ValueData::BigInt(ref b)) => {
                Self::bigint(a.clone() / b.clone())
            }
            (a, b) => Self::rational(a.to_number() / b.to_number()),
        }
    }
}
impl Rem for Value {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::BigInt(ref a), ValueData::BigInt(ref b)) => {
                Self::bigint(a.clone() % b.clone())
            }
            (a, b) => Self::rational(a.to_number() % b.to_number()),
        }
    }
}
impl BitAnd for Value {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::BigInt(ref a), ValueData::BigInt(ref b)) => {
                Self::bigint(a.clone() & b.clone())
            }
            (a, b) => Self::integer(a.to_integer() & b.to_integer()),
        }
    }
}
impl BitOr for Value {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::BigInt(ref a), ValueData::BigInt(ref b)) => {
                Self::bigint(a.clone() | b.clone())
            }
            (a, b) => Self::integer(a.to_integer() | b.to_integer()),
        }
    }
}
impl BitXor for Value {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::BigInt(ref a), ValueData::BigInt(ref b)) => {
                Self::bigint(a.clone() ^ b.clone())
            }
            (a, b) => Self::integer(a.to_integer() ^ b.to_integer()),
        }
    }
}

impl Shl for Value {
    type Output = Self;
    fn shl(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::BigInt(ref a), ValueData::BigInt(ref b)) => {
                Self::bigint(a.clone() << b.clone())
            }
            (a, b) => Self::integer(a.to_integer() << b.to_integer()),
        }
    }
}
impl Shr for Value {
    type Output = Self;
    fn shr(self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::BigInt(ref a), ValueData::BigInt(ref b)) => {
                Self::bigint(a.clone() >> b.clone())
            }
            (a, b) => Self::integer(a.to_integer() >> b.to_integer()),
        }
    }
}
impl Not for Value {
    type Output = Self;
    fn not(self) -> Self {
        Self::boolean(!self.is_true())
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self.data() {
            ValueData::Object(_) | ValueData::Symbol(_) | ValueData::Undefined => {
                Self::rational(NAN)
            }
            ValueData::String(ref str) => Self::rational(match f64::from_str(str) {
                Ok(num) => -num,
                Err(_) => NAN,
            }),
            ValueData::Rational(num) => Self::rational(-num),
            ValueData::Integer(num) => Self::rational(-f64::from(*num)),
            ValueData::Boolean(true) => Self::integer(1),
            ValueData::Boolean(false) | ValueData::Null => Self::integer(0),
            ValueData::BigInt(ref num) => Self::bigint(-num.clone()),
        }
    }
}
