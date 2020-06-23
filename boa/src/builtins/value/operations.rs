use super::*;

impl Value {
    #[inline]
    pub fn add(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::String(ref s), ref o) => {
                Self::string(format!("{}{}", s.clone(), &o.to_string()))
            }
            (Self::BigInt(ref n1), Self::BigInt(ref n2)) => {
                Self::bigint(n1.as_inner().clone() + n2.as_inner().clone())
            }
            (ref s, Self::String(ref o)) => Self::string(format!("{}{}", s.to_string(), o)),
            (ref s, ref o) => Self::rational(s.to_number() + o.to_number()),
        })
    }

    #[inline]
    pub fn sub(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() - b.as_inner().clone())
            }
            (a, b) => Self::rational(a.to_number() - b.to_number()),
        })
    }

    #[inline]
    pub fn mul(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() * b.as_inner().clone())
            }
            (a, b) => Self::rational(a.to_number() * b.to_number()),
        })
    }

    #[inline]
    pub fn div(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() / b.as_inner().clone())
            }
            (a, b) => Self::rational(a.to_number() / b.to_number()),
        })
    }

    #[inline]
    pub fn rem(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() % b.as_inner().clone())
            }
            (a, b) => Self::rational(a.to_number() % b.to_number()),
        })
    }

    #[inline]
    pub fn pow(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => Self::bigint(a.as_inner().clone().pow(b)),
            (a, b) => Self::rational(a.to_number().powf(b.to_number())),
        })
    }

    #[inline]
    pub fn bitand(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() & b.as_inner().clone())
            }
            (a, b) => Self::rational(a.to_integer() & b.to_integer()),
        })
    }

    #[inline]
    pub fn bitor(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() | b.as_inner().clone())
            }
            (a, b) => Self::integer(a.to_integer() | b.to_integer()),
        })
    }

    #[inline]
    pub fn bitxor(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() | b.as_inner().clone())
            }
            (a, b) => Self::integer(a.to_integer() | b.to_integer()),
        })
    }

    #[inline]
    pub fn shl(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() << b.as_inner().clone())
            }
            (a, b) => Self::integer(a.to_integer() << b.to_integer()),
        })
    }

    #[inline]
    pub fn shr(&self, other: &Self, _: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() >> b.as_inner().clone())
            }
            (a, b) => Self::integer(a.to_integer() >> b.to_integer()),
        })
    }

    #[inline]
    pub fn ushr(&self, other: &Self, interpreter: &mut Interpreter) -> ResultValue {
        // FIXME: Unsigned shift right
        self.shr(other, interpreter)
    }

    #[inline]
    pub fn neg(&self, _: &mut Interpreter) -> ResultValue {
        Ok(match *self {
            Self::Object(_) | Self::Symbol(_) | Self::Undefined => Self::rational(NAN),
            Self::String(ref str) => Self::rational(match f64::from_str(str) {
                Ok(num) => -num,
                Err(_) => NAN,
            }),
            Self::Rational(num) => Self::rational(-num),
            Self::Integer(num) => Self::rational(-f64::from(num)),
            Self::Boolean(true) => Self::integer(1),
            Self::Boolean(false) | Self::Null => Self::integer(0),
            Self::BigInt(ref num) => Self::bigint(-num.as_inner().clone()),
        })
    }

    #[inline]
    pub fn not(&self, _: &mut Interpreter) -> ResultValue {
        Ok(Self::boolean(!self.to_boolean()))
    }
}
