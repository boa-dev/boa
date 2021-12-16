use super::*;
use crate::builtins::number::{f64_to_int32, f64_to_uint32, Number};

impl JsValue {
    #[inline]
    pub fn add(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new(f64::from(*x) + f64::from(*y)),
            (Self::Rational(x), Self::Rational(y)) => Self::new(x + y),
            (Self::Integer(x), Self::Rational(y)) => Self::new(f64::from(*x) + y),
            (Self::Rational(x), Self::Integer(y)) => Self::new(x + f64::from(*y)),

            (Self::String(ref x), Self::String(ref y)) => Self::from(JsString::concat(x, y)),
            (Self::String(ref x), y) => Self::from(JsString::concat(x, y.to_string(context)?)),
            (x, Self::String(ref y)) => Self::from(JsString::concat(x.to_string(context)?, y)),
            (Self::BigInt(ref x), Self::BigInt(ref y)) => Self::new(JsBigInt::add(x, y)),

            // Slow path:
            (_, _) => match (
                self.to_primitive(context, PreferredType::Default)?,
                other.to_primitive(context, PreferredType::Default)?,
            ) {
                (Self::String(ref x), ref y) => {
                    Self::from(JsString::concat(x, y.to_string(context)?))
                }
                (ref x, Self::String(ref y)) => {
                    Self::from(JsString::concat(x.to_string(context)?, y))
                }
                (x, y) => match (x.to_numeric(context)?, y.to_numeric(context)?) {
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
            },
        })
    }

    #[inline]
    pub fn sub(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new(f64::from(*x) - f64::from(*y)),
            (Self::Rational(x), Self::Rational(y)) => Self::new(x - y),
            (Self::Integer(x), Self::Rational(y)) => Self::new(f64::from(*x) - y),
            (Self::Rational(x), Self::Integer(y)) => Self::new(x - f64::from(*y)),

            (Self::BigInt(ref x), Self::BigInt(ref y)) => Self::new(JsBigInt::sub(x, y)),

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
    pub fn mul(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new(f64::from(*x) * f64::from(*y)),
            (Self::Rational(x), Self::Rational(y)) => Self::new(x * y),
            (Self::Integer(x), Self::Rational(y)) => Self::new(f64::from(*x) * y),
            (Self::Rational(x), Self::Integer(y)) => Self::new(x * f64::from(*y)),

            (Self::BigInt(ref x), Self::BigInt(ref y)) => Self::new(JsBigInt::mul(x, y)),

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
    pub fn div(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new(f64::from(*x) / f64::from(*y)),
            (Self::Rational(x), Self::Rational(y)) => Self::new(x / y),
            (Self::Integer(x), Self::Rational(y)) => Self::new(f64::from(*x) / y),
            (Self::Rational(x), Self::Integer(y)) => Self::new(x / f64::from(*y)),

            (Self::BigInt(ref x), Self::BigInt(ref y)) => {
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
    pub fn rem(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => {
                if *y == 0 {
                    Self::nan()
                } else {
                    match x % *y {
                        rem if rem == 0 && *x < 0 => Self::new(-0.0),
                        rem => Self::new(rem),
                    }
                }
            }
            (Self::Rational(x), Self::Rational(y)) => Self::new((x % y).copysign(*x)),
            (Self::Integer(x), Self::Rational(y)) => {
                let x = f64::from(*x);
                Self::new((x % y).copysign(x))
            }

            (Self::Rational(x), Self::Integer(y)) => Self::new((x % f64::from(*y)).copysign(*x)),

            (Self::BigInt(ref x), Self::BigInt(ref y)) => {
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
    pub fn pow(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new(f64::from(*x).powi(*y)),
            (Self::Rational(x), Self::Rational(y)) => Self::new(x.powf(*y)),
            (Self::Integer(x), Self::Rational(y)) => Self::new(f64::from(*x).powf(*y)),
            (Self::Rational(x), Self::Integer(y)) => Self::new(x.powi(*y)),

            (Self::BigInt(ref a), Self::BigInt(ref b)) => Self::new(JsBigInt::pow(a, b, context)?),

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
    pub fn bitand(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new(x & y),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::new(f64_to_int32(*x) & f64_to_int32(*y))
            }
            (Self::Integer(x), Self::Rational(y)) => Self::new(x & f64_to_int32(*y)),
            (Self::Rational(x), Self::Integer(y)) => Self::new(f64_to_int32(*x) & y),

            (Self::BigInt(ref x), Self::BigInt(ref y)) => Self::new(JsBigInt::bitand(x, y)),

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
    pub fn bitor(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new(x | y),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::new(f64_to_int32(*x) | f64_to_int32(*y))
            }
            (Self::Integer(x), Self::Rational(y)) => Self::new(x | f64_to_int32(*y)),
            (Self::Rational(x), Self::Integer(y)) => Self::new(f64_to_int32(*x) | y),

            (Self::BigInt(ref x), Self::BigInt(ref y)) => Self::new(JsBigInt::bitor(x, y)),

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
    pub fn bitxor(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new(x ^ y),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::new(f64_to_int32(*x) ^ f64_to_int32(*y))
            }
            (Self::Integer(x), Self::Rational(y)) => Self::new(x ^ f64_to_int32(*y)),
            (Self::Rational(x), Self::Integer(y)) => Self::new(f64_to_int32(*x) ^ y),

            (Self::BigInt(ref x), Self::BigInt(ref y)) => Self::new(JsBigInt::bitxor(x, y)),

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
    pub fn shl(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new(x.wrapping_shl(*y as u32)),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::new(f64_to_int32(*x).wrapping_shl(f64_to_uint32(*y)))
            }
            (Self::Integer(x), Self::Rational(y)) => Self::new(x.wrapping_shl(f64_to_uint32(*y))),
            (Self::Rational(x), Self::Integer(y)) => {
                Self::new(f64_to_int32(*x).wrapping_shl(*y as u32))
            }

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
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
    pub fn shr(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new(x.wrapping_shr(*y as u32)),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::new(f64_to_int32(*x).wrapping_shr(f64_to_uint32(*y)))
            }
            (Self::Integer(x), Self::Rational(y)) => Self::new(x.wrapping_shr(f64_to_uint32(*y))),
            (Self::Rational(x), Self::Integer(y)) => {
                Self::new(f64_to_int32(*x).wrapping_shr(*y as u32))
            }

            (Self::BigInt(ref a), Self::BigInt(ref b)) => {
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
    pub fn ushr(&self, other: &Self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match (self, other) {
            // Fast path:
            (Self::Integer(x), Self::Integer(y)) => Self::new((*x as u32).wrapping_shr(*y as u32)),
            (Self::Rational(x), Self::Rational(y)) => {
                Self::new(f64_to_uint32(*x).wrapping_shr(f64_to_uint32(*y)))
            }
            (Self::Integer(x), Self::Rational(y)) => {
                Self::new((*x as u32).wrapping_shr(f64_to_uint32(*y)))
            }
            (Self::Rational(x), Self::Integer(y)) => {
                Self::new(f64_to_uint32(*x).wrapping_shr(*y as u32))
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
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-instanceofoperator
    #[inline]
    pub fn instance_of(&self, target: &JsValue, context: &mut Context) -> JsResult<bool> {
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
                JsValue::ordinary_has_instance(target, self, context)
            }
            None => {
                // 4. If IsCallable(target) is false, throw a TypeError exception.
                context.throw_type_error("right-hand side of 'instanceof' is not callable")
            }
        }
    }

    #[inline]
    pub fn neg(&self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match *self {
            Self::Symbol(_) | Self::Undefined => Self::new(f64::NAN),
            Self::Object(_) => Self::new(match self.to_numeric_number(context) {
                Ok(num) => -num,
                Err(_) => f64::NAN,
            }),
            Self::String(ref str) => Self::new(match f64::from_str(str) {
                Ok(num) => -num,
                Err(_) => f64::NAN,
            }),
            Self::Rational(num) => Self::new(-num),
            Self::Integer(num) if num == 0 => Self::new(-f64::from(0)),
            Self::Integer(num) => Self::new(-num),
            Self::Boolean(true) => Self::new(1),
            Self::Boolean(false) | Self::Null => Self::new(0),
            Self::BigInt(ref x) => Self::new(JsBigInt::neg(x)),
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
        context: &mut Context,
    ) -> JsResult<AbstractRelation> {
        Ok(match (self, other) {
            // Fast path (for some common operations):
            (Self::Integer(x), Self::Integer(y)) => (x < y).into(),
            (Self::Integer(x), Self::Rational(y)) => Number::less_than(f64::from(*x), *y),
            (Self::Rational(x), Self::Integer(y)) => Number::less_than(*x, f64::from(*y)),
            (Self::Rational(x), Self::Rational(y)) => Number::less_than(*x, *y),
            (Self::BigInt(ref x), Self::BigInt(ref y)) => (x < y).into(),

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

                match (px, py) {
                    (Self::String(ref x), Self::String(ref y)) => {
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
                    (Self::BigInt(ref x), Self::String(ref y)) => {
                        if let Some(y) = JsBigInt::from_string(y) {
                            (*x < y).into()
                        } else {
                            AbstractRelation::Undefined
                        }
                    }
                    (Self::String(ref x), Self::BigInt(ref y)) => {
                        if let Some(x) = JsBigInt::from_string(x) {
                            (x < *y).into()
                        } else {
                            AbstractRelation::Undefined
                        }
                    }
                    (px, py) => match (px.to_numeric(context)?, py.to_numeric(context)?) {
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
                            (*x < JsBigInt::try_from(n).unwrap()).into()
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
                            (JsBigInt::try_from(n).unwrap() < *y).into()
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
            AbstractRelation::True
        } else {
            AbstractRelation::False
        }
    }
}
