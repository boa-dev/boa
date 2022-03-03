use super::{
    Context, FromStr, JsBigInt, JsResult, JsString, JsValue, JsVariant, Numeric, PreferredType,
    WellKnownSymbols,
};
use crate::builtins::number::{f64_to_int32, f64_to_uint32, Number};

impl JsValue {
    #[inline]
    pub fn add(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => {
                Self::new(f64::from(x) + f64::from(y))
            }
            (JsVariant::Rational(x), JsVariant::Rational(y)) => Self::new(x + y),
            (JsVariant::Integer(x), JsVariant::Rational(y)) => Self::new(f64::from(x) + y),
            (JsVariant::Rational(x), JsVariant::Integer(y)) => Self::new(x + f64::from(y)),

            (JsVariant::String(ref x), JsVariant::String(ref y)) => {
                Self::from(JsString::concat(x, y))
            }
            (JsVariant::String(ref x), _) => {
                Self::new(JsString::concat(x, other.to_string(context)?))
            }
            (_, JsVariant::String(ref y)) => {
                Self::new(JsString::concat(self.to_string(context)?, y))
            }
            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => Self::new(JsBigInt::add(x, y)),

            // Slow path:
            (_, _) => {
                let x = self.to_primitive(context, PreferredType::Default)?;
                let y = other.to_primitive(context, PreferredType::Default)?;
                match (x.variant(), y.variant()) {
                    (JsVariant::String(ref x), _) => {
                        Self::from(JsString::concat(x, y.to_string(context)?))
                    }
                    (_, JsVariant::String(ref y)) => {
                        Self::from(JsString::concat(x.to_string(context)?, y))
                    }
                    (_, _) => match (x.to_numeric(context)?, y.to_numeric(context)?) {
                        (Numeric::Number(x), Numeric::Number(y)) => Self::new(x + y),
                        (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                            Self::new(JsBigInt::add(x, y))
                        }
                        (_, _) => {
                            return context.throw_type_error(
                                "cannot mix BigInt and other types, use explicit conversions",
                            )
                        }
                    },
                }
            }
        })
    }

    #[inline]
    pub fn sub(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => {
                Self::new(f64::from(x) - f64::from(y))
            }
            (JsVariant::Rational(x), JsVariant::Rational(y)) => Self::new(x - y),
            (JsVariant::Integer(x), JsVariant::Rational(y)) => Self::new(f64::from(x) - y),
            (JsVariant::Rational(x), JsVariant::Integer(y)) => Self::new(x - f64::from(y)),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => Self::new(JsBigInt::sub(x, y)),

            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(a), Numeric::Number(b)) => Self::new(a - b),
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => Self::new(JsBigInt::sub(x, y)),
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn mul(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => {
                Self::new(f64::from(x) * f64::from(y))
            }
            (JsVariant::Rational(x), JsVariant::Rational(y)) => Self::new(x * y),
            (JsVariant::Integer(x), JsVariant::Rational(y)) => Self::new(f64::from(x) * y),
            (JsVariant::Rational(x), JsVariant::Integer(y)) => Self::new(x * f64::from(y)),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => Self::new(JsBigInt::mul(x, y)),

            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(a), Numeric::Number(b)) => Self::new(a * b),
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => Self::new(JsBigInt::mul(x, y)),
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn div(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => {
                Self::new(f64::from(x) / f64::from(y))
            }
            (JsVariant::Rational(x), JsVariant::Rational(y)) => Self::new(x / y),
            (JsVariant::Integer(x), JsVariant::Rational(y)) => Self::new(f64::from(x) / y),
            (JsVariant::Rational(x), JsVariant::Integer(y)) => Self::new(x / f64::from(y)),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                if y.is_zero() {
                    return context.throw_range_error("BigInt division by zero");
                }
                Self::new(JsBigInt::div(x, y))
            }

            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(a), Numeric::Number(b)) => Self::new(a / b),
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    if y.is_zero() {
                        return context.throw_range_error("BigInt division by zero");
                    }
                    Self::new(JsBigInt::div(x, y))
                }
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn rem(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => {
                if y == 0 {
                    Self::nan()
                } else {
                    match x % y {
                        rem if rem == 0 && x < 0 => Self::new(-0.0),
                        rem => Self::new(rem),
                    }
                }
            }
            (JsVariant::Rational(x), JsVariant::Rational(y)) => Self::new((x % y).copysign(x)),
            (JsVariant::Integer(x), JsVariant::Rational(y)) => {
                let x = f64::from(x);
                Self::new((x % y).copysign(x))
            }
            (JsVariant::Rational(x), JsVariant::Integer(y)) => {
                Self::new((x % f64::from(y)).copysign(x))
            }
            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => {
                if y.is_zero() {
                    return context.throw_range_error("BigInt division by zero");
                }
                Self::new(JsBigInt::rem(x, y))
            }
            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(a), Numeric::Number(b)) => Self::new(a % b),
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    if y.is_zero() {
                        return context.throw_range_error("BigInt division by zero");
                    }
                    Self::new(JsBigInt::rem(x, y))
                }
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn pow(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => Self::new(f64::from(x).powi(y)),
            (JsVariant::Rational(x), JsVariant::Rational(y)) => Self::new(x.powf(y)),
            (JsVariant::Integer(x), JsVariant::Rational(y)) => Self::new(f64::from(x).powf(y)),
            (JsVariant::Rational(x), JsVariant::Integer(y)) => Self::new(x.powi(y)),

            (JsVariant::BigInt(a), JsVariant::BigInt(b)) => {
                Self::new(JsBigInt::pow(a, b, context)?)
            }

            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(a), Numeric::Number(b)) => Self::new(a.powf(b)),
                (Numeric::BigInt(ref a), Numeric::BigInt(ref b)) => {
                    Self::new(JsBigInt::pow(a, b, context)?)
                }
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn bitand(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => Self::new(x & y),
            (JsVariant::Rational(x), JsVariant::Rational(y)) => {
                Self::new(f64_to_int32(x) & f64_to_int32(y))
            }
            (JsVariant::Integer(x), JsVariant::Rational(y)) => Self::new(x & f64_to_int32(y)),
            (JsVariant::Rational(x), JsVariant::Integer(y)) => Self::new(f64_to_int32(x) & y),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => Self::new(JsBigInt::bitand(x, y)),

            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(a), Numeric::Number(b)) => {
                    Self::new(f64_to_int32(a) & f64_to_int32(b))
                }
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    Self::new(JsBigInt::bitand(x, y))
                }
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn bitor(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => Self::new(x | y),
            (JsVariant::Rational(x), JsVariant::Rational(y)) => {
                Self::new(f64_to_int32(x) | f64_to_int32(y))
            }
            (JsVariant::Integer(x), JsVariant::Rational(y)) => Self::new(x | f64_to_int32(y)),
            (JsVariant::Rational(x), JsVariant::Integer(y)) => Self::new(f64_to_int32(x) | y),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => Self::new(JsBigInt::bitor(x, y)),

            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(a), Numeric::Number(b)) => {
                    Self::new(f64_to_int32(a) | f64_to_int32(b))
                }
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    Self::new(JsBigInt::bitor(x, y))
                }
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn bitxor(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => Self::new(x ^ y),
            (JsVariant::Rational(x), JsVariant::Rational(y)) => {
                Self::new(f64_to_int32(x) ^ f64_to_int32(y))
            }
            (JsVariant::Integer(x), JsVariant::Rational(y)) => Self::new(x ^ f64_to_int32(y)),
            (JsVariant::Rational(x), JsVariant::Integer(y)) => Self::new(f64_to_int32(x) ^ y),

            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => Self::new(JsBigInt::bitxor(x, y)),

            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(a), Numeric::Number(b)) => {
                    Self::new(f64_to_int32(a) ^ f64_to_int32(b))
                }
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    Self::new(JsBigInt::bitxor(x, y))
                }
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn shl(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => Self::new(x.wrapping_shl(y as u32)),
            (JsVariant::Rational(x), JsVariant::Rational(y)) => {
                Self::new(f64_to_int32(x).wrapping_shl(f64_to_uint32(y)))
            }
            (JsVariant::Integer(x), JsVariant::Rational(y)) => {
                Self::new(x.wrapping_shl(f64_to_uint32(y)))
            }
            (JsVariant::Rational(x), JsVariant::Integer(y)) => {
                Self::new(f64_to_int32(x).wrapping_shl(y as u32))
            }

            (JsVariant::BigInt(a), JsVariant::BigInt(b)) => {
                Self::new(JsBigInt::shift_left(a, b, context)?)
            }

            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(x), Numeric::Number(y)) => {
                    Self::new(f64_to_int32(x).wrapping_shl(f64_to_uint32(y)))
                }
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    Self::new(JsBigInt::shift_left(x, y, context)?)
                }
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn shr(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => Self::new(x.wrapping_shr(y as u32)),
            (JsVariant::Rational(x), JsVariant::Rational(y)) => {
                Self::new(f64_to_int32(x).wrapping_shr(f64_to_uint32(y)))
            }
            (JsVariant::Integer(x), JsVariant::Rational(y)) => {
                Self::new(x.wrapping_shr(f64_to_uint32(y)))
            }
            (JsVariant::Rational(x), JsVariant::Integer(y)) => {
                Self::new(f64_to_int32(x).wrapping_shr(y as u32))
            }

            (JsVariant::BigInt(a), JsVariant::BigInt(b)) => {
                Self::new(JsBigInt::shift_right(a, b, context)?)
            }

            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(x), Numeric::Number(y)) => {
                    Self::new(f64_to_int32(x).wrapping_shr(f64_to_uint32(y)))
                }
                (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => {
                    Self::new(JsBigInt::shift_right(x, y, context)?)
                }
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    #[inline]
    pub fn ushr(&self, other: &Self, context: &mut Context) -> JsResult<Self> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path:
            (JsVariant::Integer(x), JsVariant::Integer(y)) => {
                Self::new((x as u32).wrapping_shr(y as u32))
            }
            (JsVariant::Rational(x), JsVariant::Rational(y)) => {
                Self::new(f64_to_uint32(x).wrapping_shr(f64_to_uint32(y)))
            }
            (JsVariant::Integer(x), JsVariant::Rational(y)) => {
                Self::new((x as u32).wrapping_shr(f64_to_uint32(y)))
            }
            (JsVariant::Rational(x), JsVariant::Integer(y)) => {
                Self::new(f64_to_uint32(x).wrapping_shr(y as u32))
            }

            // Slow path:
            (_, _) => match (self.to_numeric(context)?, other.to_numeric(context)?) {
                (Numeric::Number(x), Numeric::Number(y)) => {
                    Self::new(f64_to_uint32(x).wrapping_shr(f64_to_uint32(y)))
                }
                (Numeric::BigInt(_), Numeric::BigInt(_)) => {
                    return context
                        .throw_type_error("BigInts have no unsigned right shift, use >> instead");
                }
                (_, _) => {
                    return context.throw_type_error(
                        "cannot mix BigInt and other types, use explicit conversions",
                    );
                }
            },
        })
    }

    /// Abstract operation `InstanceofOperator ( V, target )`
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-instanceofoperator
    #[inline]
    pub fn instance_of(&self, target: &Self, context: &mut Context) -> JsResult<bool> {
        // 1. If Type(target) is not Object, throw a TypeError exception.
        if !target.is_object() {
            return context.throw_type_error(format!(
                "right-hand side of 'instanceof' should be an object, got {}",
                target.type_of()
            ));
        }

        // 2. Let instOfHandler be ? GetMethod(target, @@hasInstance).
        match target.get_method(WellKnownSymbols::has_instance(), context)? {
            // 3. If instOfHandler is not undefined, then
            Some(instance_of_handler) => {
                // a. Return ! ToBoolean(? Call(instOfHandler, target, « V »)).
                Ok(instance_of_handler
                    .call(target, std::slice::from_ref(self), context)?
                    .to_boolean())
            }
            None if target.is_callable() => {
                // 5. Return ? OrdinaryHasInstance(target, V).
                Self::ordinary_has_instance(target, self, context)
            }
            None => {
                // 4. If IsCallable(target) is false, throw a TypeError exception.
                context.throw_type_error("right-hand side of 'instanceof' is not callable")
            }
        }
    }

    #[inline]
    pub fn neg(&self, context: &mut Context) -> JsResult<Self> {
        Ok(match self.variant() {
            JsVariant::Symbol(_) | JsVariant::Undefined => Self::new(f64::NAN),
            JsVariant::Object(_) => Self::new(match self.to_numeric_number(context) {
                Ok(num) => -num,
                Err(_) => f64::NAN,
            }),
            JsVariant::String(str) => Self::new(match f64::from_str(str) {
                Ok(num) => -num,
                Err(_) => f64::NAN,
            }),
            JsVariant::Rational(num) => Self::new(-num),
            JsVariant::Integer(num) if num == 0 => Self::new(-f64::from(0)),
            JsVariant::Integer(num) => Self::new(-num),
            JsVariant::Boolean(true) => Self::new(1),
            JsVariant::Boolean(false) | JsVariant::Null => Self::new(0),
            JsVariant::BigInt(x) => Self::new(JsBigInt::neg(x)),
        })
    }

    #[inline]
    pub fn not(&self, _: &mut Context) -> JsResult<bool> {
        Ok(!self.to_boolean())
    }

    /// Abstract relational comparison
    ///
    /// The comparison `x < y`, where `x` and `y` are values, produces `true`, `false`,
    /// or `undefined` (which indicates that at least one operand is `NaN`).
    ///
    /// In addition to `x` and `y` the algorithm takes a Boolean flag named `LeftFirst` as a parameter.
    /// The flag is used to control the order in which operations with potentially visible side-effects
    /// are performed upon `x` and `y`. It is necessary because ECMAScript specifies left to right evaluation
    /// of expressions. The default value of `LeftFirst` is `true` and indicates that the `x` parameter
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
        context: &mut Context,
    ) -> JsResult<AbstractRelation> {
        Ok(match (self.variant(), other.variant()) {
            // Fast path (for some common operations):
            (JsVariant::Integer(x), JsVariant::Integer(y)) => (x < y).into(),
            (JsVariant::Integer(x), JsVariant::Rational(y)) => Number::less_than(f64::from(x), y),
            (JsVariant::Rational(x), JsVariant::Integer(y)) => Number::less_than(x, f64::from(y)),
            (JsVariant::Rational(x), JsVariant::Rational(y)) => Number::less_than(x, y),
            (JsVariant::BigInt(x), JsVariant::BigInt(y)) => (x < y).into(),

            // Slow path:
            (_, _) => {
                let (px, py) = if left_first {
                    let px = self.to_primitive(context, PreferredType::Number)?;
                    let py = other.to_primitive(context, PreferredType::Number)?;
                    (px, py)
                } else {
                    // NOTE: The order of evaluation needs to be reversed to preserve left to right evaluation.
                    let py = other.to_primitive(context, PreferredType::Number)?;
                    let px = self.to_primitive(context, PreferredType::Number)?;
                    (px, py)
                };

                match (px.variant(), py.variant()) {
                    (JsVariant::String(x), JsVariant::String(y)) => {
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
                    (JsVariant::BigInt(x), JsVariant::String(y)) => {
                        if let Some(y) = JsBigInt::from_string(y) {
                            (*x < y).into()
                        } else {
                            AbstractRelation::Undefined
                        }
                    }
                    (JsVariant::String(x), JsVariant::BigInt(y)) => {
                        if let Some(x) = JsBigInt::from_string(x) {
                            (x < *y).into()
                        } else {
                            AbstractRelation::Undefined
                        }
                    }
                    (_, _) => match (px.to_numeric(context)?, py.to_numeric(context)?) {
                        (Numeric::Number(x), Numeric::Number(y)) => Number::less_than(x, y),
                        (Numeric::BigInt(ref x), Numeric::BigInt(ref y)) => (x < y).into(),
                        (Numeric::BigInt(ref x), Numeric::Number(y)) => {
                            if y.is_nan() {
                                return Ok(AbstractRelation::Undefined);
                            }
                            if y.is_infinite() {
                                return Ok(y.is_sign_positive().into());
                            }
                            let n = if y.is_sign_negative() {
                                y.floor()
                            } else {
                                y.ceil()
                            };
                            (*x < JsBigInt::try_from(n).expect("invalid conversion to BigInt"))
                                .into()
                        }
                        (Numeric::Number(x), Numeric::BigInt(ref y)) => {
                            if x.is_nan() {
                                return Ok(AbstractRelation::Undefined);
                            }
                            if x.is_infinite() {
                                return Ok(x.is_sign_negative().into());
                            }
                            let n = if x.is_sign_negative() {
                                x.floor()
                            } else {
                                x.ceil()
                            };
                            (JsBigInt::try_from(n).expect("invalid conversion to BigInt") < *y)
                                .into()
                        }
                    },
                }
            }
        })
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
    pub fn lt(&self, other: &Self, context: &mut Context) -> JsResult<bool> {
        match self.abstract_relation(other, true, context)? {
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
    pub fn le(&self, other: &Self, context: &mut Context) -> JsResult<bool> {
        match other.abstract_relation(self, false, context)? {
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
    pub fn gt(&self, other: &Self, context: &mut Context) -> JsResult<bool> {
        match other.abstract_relation(self, false, context)? {
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
    pub fn ge(&self, other: &Self, context: &mut Context) -> JsResult<bool> {
        match self.abstract_relation(other, true, context)? {
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
            Self::True
        } else {
            Self::False
        }
    }
}
