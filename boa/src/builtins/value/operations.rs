use super::*;
use crate::builtins::number::Number;
use crate::exec::PreferredType;

impl Value {
    #[inline]
    pub fn add(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(
            match (
                ctx.to_primitive(self, PreferredType::Default)?,
                ctx.to_primitive(other, PreferredType::Default)?,
            ) {
                (Self::String(ref x), ref y) => Self::string(format!("{}{}", x, ctx.to_string(y)?)),
                (ref x, Self::String(ref y)) => Self::string(format!("{}{}", ctx.to_string(x)?, y)),
                (ref x, ref y) => match (ctx.to_numeric(x)?, ctx.to_numeric(y)?) {
                    (Self::BigInt(ref n1), Self::BigInt(ref n2)) => {
                        Self::bigint(n1.as_inner().clone() + n2.as_inner().clone())
                    }
                    (Self::Rational(x), Self::Rational(y)) => Self::rational(x + y),
                    (_, _) => {
                        return ctx.throw_type_error(
                            "cannot mix BigInt and other types, use explicit conversions",
                        )
                    }
                },
            },
        )
    }

    #[inline]
    pub fn sub(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() - b.as_inner().clone())
            }
            (Self::Rational(a), Self::Rational(b)) => Self::rational(a - b),
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
    }

    #[inline]
    pub fn mul(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::Rational(a), Self::Rational(b)) => Self::rational(a * b),
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() * b.as_inner().clone())
            }
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
    }

    #[inline]
    pub fn div(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::Rational(a), Self::Rational(b)) => Self::rational(a / b),
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() / b.as_inner().clone())
            }
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
    }

    #[inline]
    pub fn rem(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::Rational(a), Self::Rational(b)) => Self::rational(a % b),
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() % b.as_inner().clone())
            }
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
    }

    #[inline]
    pub fn pow(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::Rational(a), Self::Rational(b)) => Self::rational(a.powf(b)),
            (Self::BigInt(ref a), Self::BigInt(ref b)) => Self::bigint(a.as_inner().clone().pow(b)),
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
    }

    #[inline]
    pub fn bitand(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::Rational(a), Self::Rational(b)) => {
                Self::integer(Number::new(a).to_int32() & Number::new(b).to_int32())
            }
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() & b.as_inner().clone())
            }
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
    }

    #[inline]
    pub fn bitor(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::Rational(a), Self::Rational(b)) => {
                Self::integer(Number::new(a).to_int32() | Number::new(b).to_int32())
            }
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() | b.as_inner().clone())
            }
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
    }

    #[inline]
    pub fn bitxor(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::Rational(a), Self::Rational(b)) => {
                Self::integer(Number::new(a).to_int32() ^ Number::new(b).to_int32())
            }
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() ^ b.as_inner().clone())
            }
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
    }

    #[inline]
    pub fn shl(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::Rational(a), Self::Rational(b)) => Self::integer(
                Number::new(a)
                    .to_int32()
                    .wrapping_shl(Number::new(b).to_uint32()),
            ),
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() << b.as_inner().clone())
            }
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
    }

    #[inline]
    pub fn shr(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::Rational(a), Self::Rational(b)) => Self::integer(
                Number::new(a)
                    .to_int32()
                    .wrapping_shr(Number::new(b).to_uint32()),
            ),
            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() >> b.as_inner().clone())
            }
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
    }

    #[inline]
    pub fn ushr(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
            (Self::Rational(a), Self::Rational(b)) => Self::number(
                Number::new(a)
                    .to_uint32()
                    .wrapping_shr(Number::new(b).to_uint32()),
            ),
            (Self::BigInt(_), Self::BigInt(_)) => {
                return ctx
                    .throw_type_error("BigInts have no unsigned right shift, use >> instead");
            }
            (_, _) => {
                return ctx.throw_type_error(
                    "cannot mix BigInt and other types, use explicit conversions",
                );
            }
        })
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
