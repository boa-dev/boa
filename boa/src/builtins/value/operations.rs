use super::*;
use crate::builtins::number::{f64_to_int32, f64_to_uint32, Number};
use crate::exec::PreferredType;

impl Value {
    #[inline]
    pub fn add(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::rational(f64::from(*x) + f64::from(*y)),
            (Self::Rational(x), Self::Rational(y)) => Self::rational(x + y),
            (Self::Integer(x), Self::Rational(y)) => Self::rational(f64::from(*x) + y),
            (Self::Rational(x), Self::Integer(y)) => Self::rational(x + f64::from(*y)),

            (Self::String(ref x), ref y) => Self::string(format!("{}{}", x, ctx.to_string(y)?)),
            (ref x, Self::String(ref y)) => Self::string(format!("{}{}", ctx.to_string(x)?, y)),
            (Self::BigInt(ref n1), Self::BigInt(ref n2)) => {
                Self::bigint(n1.as_inner().clone() + n2.as_inner().clone())
            }

            // Slow path:
            (_, _) => match (
                ctx.to_primitive(self, PreferredType::Default)?,
                ctx.to_primitive(other, PreferredType::Default)?,
            ) {
                (Self::String(ref x), ref y) => Self::string(format!("{}{}", x, ctx.to_string(y)?)),
                (ref x, Self::String(ref y)) => Self::string(format!("{}{}", ctx.to_string(x)?, y)),
                (x, y) => match (ctx.to_numeric(&x)?, ctx.to_numeric(&y)?) {
                    (Self::Rational(x), Self::Rational(y)) => Self::rational(x + y),
                    (Self::BigInt(ref n1), Self::BigInt(ref n2)) => {
                        Self::bigint(n1.as_inner().clone() + n2.as_inner().clone())
                    }
                    (_, _) => {
                        return ctx.throw_type_error(
                            "cannot mix BigInt and other types, use explicit conversions",
                        )
                    }
                },
            },
        })
    }

    #[inline]
    pub fn sub(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::rational(f64::from(*x) - f64::from(*y)),
            (Self::Rational(x), Self::Rational(y)) => Self::rational(x - y),
            (Self::Integer(x), Self::Rational(y)) => Self::rational(f64::from(*x) - y),
            (Self::Rational(x), Self::Integer(y)) => Self::rational(x - f64::from(*y)),

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() - b.as_inner().clone())
            }

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(a), Self::Rational(b)) => Self::rational(a - b),
                (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                    Self::bigint(a.as_inner().clone() - b.as_inner().clone())
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn mul(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::rational(f64::from(*x) * f64::from(*y)),
            (Self::Rational(x), Self::Rational(y)) => Self::rational(x * y),
            (Self::Integer(x), Self::Rational(y)) => Self::rational(f64::from(*x) * y),
            (Self::Rational(x), Self::Integer(y)) => Self::rational(x * f64::from(*y)),

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() * b.as_inner().clone())
            }

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(a), Self::Rational(b)) => Self::rational(a * b),
                (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                    Self::bigint(a.as_inner().clone() * b.as_inner().clone())
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn div(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::rational(f64::from(*x) / f64::from(*y)),
            (Self::Rational(x), Self::Rational(y)) => Self::rational(x / y),
            (Self::Integer(x), Self::Rational(y)) => Self::rational(f64::from(*x) / y),
            (Self::Rational(x), Self::Integer(y)) => Self::rational(x / f64::from(*y)),

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() / b.as_inner().clone())
            }

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(a), Self::Rational(b)) => Self::rational(a / b),
                (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                    Self::bigint(a.as_inner().clone() / b.as_inner().clone())
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn rem(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::integer(x % *y),
            (Self::Rational(x), Self::Rational(y)) => Self::rational(x % y),
            (Self::Integer(x), Self::Rational(y)) => Self::rational(f64::from(*x) % y),
            (Self::Rational(x), Self::Integer(y)) => Self::rational(x % f64::from(*y)),

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() % b.as_inner().clone())
            }

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(a), Self::Rational(b)) => Self::rational(a % b),
                (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                    Self::bigint(a.as_inner().clone() % b.as_inner().clone())
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn pow(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::rational(f64::from(*x).powi(*y)),
            (Self::Rational(x), Self::Rational(y)) => Self::rational(x.powf(*y)),
            (Self::Integer(x), Self::Rational(y)) => Self::rational(f64::from(*x).powf(*y)),
            (Self::Rational(x), Self::Integer(y)) => Self::rational(x.powi(*y)),

            (Self::BigInt(ref a), Self::BigInt(ref b)) => Self::bigint(a.as_inner().clone().pow(b)),

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(a), Self::Rational(b)) => Self::rational(a.powf(b)),
                (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                    Self::bigint(a.as_inner().clone().pow(b))
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn bitand(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::integer(x & y),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::integer(f64_to_int32(*x) & f64_to_int32(*y))
            }
            (Self::Integer(x), Self::Rational(y)) => Self::integer(x & f64_to_int32(*y)),
            (Self::Rational(x), Self::Integer(y)) => Self::integer(f64_to_int32(*x) & y),

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() & b.as_inner().clone())
            }

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(a), Self::Rational(b)) => {
                    Self::integer(f64_to_int32(a) & f64_to_int32(b))
                }
                (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                    Self::bigint(a.as_inner().clone() & b.as_inner().clone())
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn bitor(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::integer(x | y),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::integer(f64_to_int32(*x) | f64_to_int32(*y))
            }
            (Self::Integer(x), Self::Rational(y)) => Self::integer(x | f64_to_int32(*y)),
            (Self::Rational(x), Self::Integer(y)) => Self::integer(f64_to_int32(*x) | y),

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() | b.as_inner().clone())
            }

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(a), Self::Rational(b)) => {
                    Self::integer(f64_to_int32(a) | f64_to_int32(b))
                }
                (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                    Self::bigint(a.as_inner().clone() | b.as_inner().clone())
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn bitxor(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::integer(x ^ y),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::integer(f64_to_int32(*x) ^ f64_to_int32(*y))
            }
            (Self::Integer(x), Self::Rational(y)) => Self::integer(x ^ f64_to_int32(*y)),
            (Self::Rational(x), Self::Integer(y)) => Self::integer(f64_to_int32(*x) ^ y),

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() ^ b.as_inner().clone())
            }

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(a), Self::Rational(b)) => {
                    Self::integer(f64_to_int32(a) ^ f64_to_int32(b))
                }
                (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                    Self::bigint(a.as_inner().clone() ^ b.as_inner().clone())
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn shl(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::integer(x.wrapping_shl(*y as u32)),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::integer(f64_to_int32(*x).wrapping_shl(f64_to_uint32(*y)))
            }
            (Self::Integer(x), Self::Rational(y)) => {
                Self::integer(x.wrapping_shl(f64_to_uint32(*y)))
            }
            (Self::Rational(x), Self::Integer(y)) => {
                Self::integer(f64_to_int32(*x).wrapping_shl(*y as u32))
            }

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() << b.as_inner().clone())
            }

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(x), Self::Rational(y)) => {
                    Self::integer(f64_to_int32(x).wrapping_shl(f64_to_uint32(y)))
                }
                (Self::BigInt(ref x), Self::BigInt(ref y)) => {
                    Self::bigint(x.as_inner().clone() << y.as_inner().clone())
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn shr(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::integer(x.wrapping_shr(*y as u32)),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::integer(f64_to_int32(*x).wrapping_shr(f64_to_uint32(*y)))
            }
            (Self::Integer(x), Self::Rational(y)) => {
                Self::integer(x.wrapping_shr(f64_to_uint32(*y)))
            }
            (Self::Rational(x), Self::Integer(y)) => {
                Self::integer(f64_to_int32(*x).wrapping_shr(*y as u32))
            }

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
                Self::bigint(a.as_inner().clone() >> b.as_inner().clone())
            }

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(x), Self::Rational(y)) => {
                    Self::integer(f64_to_int32(x).wrapping_shr(f64_to_uint32(y)))
                }
                (Self::BigInt(ref x), Self::BigInt(ref y)) => {
                    Self::bigint(x.as_inner().clone() >> y.as_inner().clone())
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn ushr(&self, other: &Self, ctx: &mut Interpreter) -> ResultValue {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => {
                Self::rational((*x as u32).wrapping_shr(*y as u32))
            }
            (Self::Rational(x), Self::Rational(y)) => {
                Self::rational(f64_to_uint32(*x).wrapping_shr(f64_to_uint32(*y)))
            }
            (Self::Integer(x), Self::Rational(y)) => {
                Self::rational((*x as u32).wrapping_shr(f64_to_uint32(*y)))
            }
            (Self::Rational(x), Self::Integer(y)) => {
                Self::rational(f64_to_uint32(*x).wrapping_shr(*y as u32))
            }

            // Slow path:
            (_, _) => match (ctx.to_numeric(self)?, ctx.to_numeric(other)?) {
                (Self::Rational(x), Self::Rational(y)) => {
                    Self::rational(f64_to_uint32(x).wrapping_shr(f64_to_uint32(y)))
                }
                (Self::BigInt(_), Self::BigInt(_)) => {
                    return ctx
                        .throw_type_error("BigInts have no unsigned right shift, use >> instead");
                }
                (_, _) => {
                    return ctx.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn neg(&self, interpreter: &mut Interpreter) -> ResultValue {
        Ok(match *self {
            Self::Symbol(_) | Self::Undefined => Self::rational(NAN),
            Self::Object(_) => Self::rational(match interpreter.to_numeric_number(self) {
                Ok(num) => -num,
                Err(_) => NAN,
            }),
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

    /// Abstract relational comparison
    ///
    /// The comparison `x < y`, where `x` and `y` are values, produces `true`, `false`,
    /// or `undefined` (which indicates that at least one operand is `NaN`).
    ///
    /// In addition to `x` and `y` the algorithm takes a Boolean flag named `LeftFirst` as a parameter.
    /// The flag is used to control the order in which operations with potentially visible side-effects
    /// are performed upon `x` and `y`. It is necessary because ECMAScript specifies left to right evaluation
    /// of expressions. The default value of LeftFirst is `true` and indicates that the `x` parameter
    /// corresponds to an expression that occurs to the left of the `y` parameter's corresponding expression.
    ///
    /// If `LeftFirst` is `false`, the reverse is the case and operations must be performed upon `y` before `x`.
    ///
    /// More Information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-abstract-relational-comparison
    pub fn abstract_relation(
        &self,
        other: &Self,
        left_first: bool,
        ctx: &mut Interpreter,
    ) -> Result<AbstractRelation, Value> {
        let (px, py) = if left_first {
            let px = ctx.to_primitive(self, PreferredType::Number)?;
            let py = ctx.to_primitive(other, PreferredType::Number)?;
            (px, py)
        } else {
            // NOTE: The order of evaluation needs to be reversed to preserve left to right evaluation.
            let py = ctx.to_primitive(other, PreferredType::Number)?;
            let px = ctx.to_primitive(self, PreferredType::Number)?;
            (px, py)
        };

        match (px, py) {
            (Value::String(ref x), Value::String(ref y)) => {
                if x.starts_with(y.as_str()) {
                    return Ok(AbstractRelation::False);
                }
                if y.starts_with(x.as_str()) {
                    return Ok(AbstractRelation::True);
                }
                for (x, y) in x.chars().zip(y.chars()) {
                    if x != y {
                        return Ok((x < y).into());
                    }
                }
                unreachable!()
            }
            (Value::BigInt(ref x), Value::String(ref y)) => {
                Ok(if let Some(y) = string_to_bigint(&y) {
                    (*x.as_inner() < y).into()
                } else {
                    AbstractRelation::Undefined
                })
            }
            (Value::String(ref x), Value::BigInt(ref y)) => {
                Ok(if let Some(x) = string_to_bigint(&x) {
                    (x < *y.as_inner()).into()
                } else {
                    AbstractRelation::Undefined
                })
            }
            (px, py) => {
                let nx = ctx.to_numeric(&px)?;
                let ny = ctx.to_numeric(&py)?;
                Ok(match (nx, ny) {
                    (Value::Integer(x), Value::Integer(y)) => (x < y).into(),
                    (Value::Integer(x), Value::Rational(y)) => Number::less_than(x.into(), y),
                    (Value::Rational(x), Value::Integer(y)) => Number::less_than(x, y.into()),
                    (Value::Rational(x), Value::Rational(y)) => Number::less_than(x, y),
                    (Value::BigInt(ref x), Value::BigInt(ref y)) => (x < y).into(),
                    (Value::BigInt(ref x), Value::Rational(y)) => {
                        if y.is_nan() {
                            return Ok(AbstractRelation::Undefined);
                        }
                        if y.is_infinite() {
                            return Ok(y.is_sign_positive().into());
                        }
                        (*x.as_inner() < BigInt::try_from(y.trunc()).unwrap()).into()
                    }
                    (Value::Rational(x), Value::BigInt(ref y)) => {
                        if x.is_nan() {
                            return Ok(AbstractRelation::Undefined);
                        }
                        if x.is_infinite() {
                            return Ok(x.is_sign_negative().into());
                        }
                        (BigInt::try_from(x.trunc()).unwrap() < *y.as_inner()).into()
                    }
                    (_, _) => unreachable!(),
                })
            }
        }
    }

    /// The less than operator (`<`) returns `true` if the left operand is less than the right operand,
    /// and `false` otherwise.
    ///
    /// More Information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Less_than
    /// [spec]: https://tc39.es/ecma262/#sec-relational-operators-runtime-semantics-evaluation
    #[inline]
    pub fn lt(&self, other: &Self, ctx: &mut Interpreter) -> Result<bool, Value> {
        match self.abstract_relation(other, true, ctx)? {
            AbstractRelation::True => Ok(true),
            AbstractRelation::False | AbstractRelation::Undefined => Ok(false),
        }
    }

    /// The less than or equal operator (`<=`) returns `true` if the left operand is less than
    /// or equal to the right operand, and `false` otherwise.
    ///
    /// More Information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Less_than_or_equal
    /// [spec]: https://tc39.es/ecma262/#sec-relational-operators-runtime-semantics-evaluation
    #[inline]
    pub fn le(&self, other: &Self, ctx: &mut Interpreter) -> Result<bool, Value> {
        match other.abstract_relation(self, false, ctx)? {
            AbstractRelation::False => Ok(true),
            AbstractRelation::True | AbstractRelation::Undefined => Ok(false),
        }
    }

    /// The greater than operator (`>`) returns `true` if the left operand is greater than
    /// the right operand, and `false` otherwise.
    ///
    /// More Information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Greater_than
    /// [spec]: https://tc39.es/ecma262/#sec-relational-operators-runtime-semantics-evaluation
    #[inline]
    pub fn gt(&self, other: &Self, ctx: &mut Interpreter) -> Result<bool, Value> {
        match other.abstract_relation(self, false, ctx)? {
            AbstractRelation::True => Ok(true),
            AbstractRelation::False | AbstractRelation::Undefined => Ok(false),
        }
    }

    /// The greater than or equal operator (`>=`) returns `true` if the left operand is greater than
    /// or equal to the right operand, and `false` otherwise.
    ///
    /// More Information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Greater_than_or_equal
    /// [spec]: https://tc39.es/ecma262/#sec-relational-operators-runtime-semantics-evaluation
    #[inline]
    pub fn ge(&self, other: &Self, ctx: &mut Interpreter) -> Result<bool, Value> {
        match self.abstract_relation(other, true, ctx)? {
            AbstractRelation::False => Ok(true),
            AbstractRelation::True | AbstractRelation::Undefined => Ok(false),
        }
    }
}

/// The result of the [Abstract Relational Comparison][arc].
///
/// Comparison `x < y`, where `x` and `y` are values.
/// It produces `true`, `false`, or `undefined`
/// (which indicates that at least one operand is `NaN`).
///
/// [arc]: https://tc39.es/ecma262/#sec-abstract-relational-comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbstractRelation {
    /// `x` is less than `y`
    True,
    /// `x` is **not** less than `y`
    False,
    /// Indicates that at least one operand is `NaN`
    Undefined,
}

impl From<bool> for AbstractRelation {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            AbstractRelation::True
        } else {
            AbstractRelation::False
        }
    }
}
