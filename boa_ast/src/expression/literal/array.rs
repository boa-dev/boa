//! Array declaration Expression.

use crate::expression::operator::assign::AssignTarget;
use crate::expression::Expression;
use crate::pattern::{ArrayPattern, ArrayPatternElement, Pattern};
use crate::try_break;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use boa_interner::{Interner, Sym, ToInternedString};
use core::ops::ControlFlow;

/// An array is an ordered collection of data (either primitive or object depending upon the
/// language).
///
/// Arrays are used to store multiple values in a single variable.
/// This is compared to a variable that can store only one value.
///
/// Each item in an array has a number attached to it, called a numeric index, that allows you
/// to access it. In JavaScript, arrays start at index zero and can be manipulated with various
/// methods.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ArrayLiteral
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ArrayLiteral {
    arr: Box<[Option<Expression>]>,
    has_trailing_comma_spread: bool,
}

impl ArrayLiteral {
    /// Creates a new array literal.
    pub fn new<A>(array: A, has_trailing_comma_spread: bool) -> Self
    where
        A: Into<Box<[Option<Expression>]>>,
    {
        Self {
            arr: array.into(),
            has_trailing_comma_spread,
        }
    }

    /// Indicates if a spread operator in the array literal has a trailing comma.
    /// This is a syntax error in some cases.
    #[must_use]
    pub const fn has_trailing_comma_spread(&self) -> bool {
        self.has_trailing_comma_spread
    }

    /// Converts this `ArrayLiteral` into an [`ArrayPattern`].
    #[must_use]
    pub fn to_pattern(&self, strict: bool) -> Option<ArrayPattern> {
        if self.has_trailing_comma_spread() {
            return None;
        }

        let mut bindings = Vec::new();
        for (i, expr) in self.arr.iter().enumerate() {
            let expr = if let Some(expr) = expr {
                expr
            } else {
                bindings.push(ArrayPatternElement::Elision);
                continue;
            };
            match expr {
                Expression::Identifier(ident) => {
                    if strict && *ident == Sym::ARGUMENTS {
                        return None;
                    }

                    bindings.push(ArrayPatternElement::SingleName {
                        ident: *ident,
                        default_init: None,
                    });
                }
                Expression::Spread(spread) => {
                    match spread.target() {
                        Expression::Identifier(ident) => {
                            bindings.push(ArrayPatternElement::SingleNameRest { ident: *ident });
                        }
                        Expression::PropertyAccess(access) => {
                            bindings.push(ArrayPatternElement::PropertyAccessRest {
                                access: access.clone(),
                            });
                        }
                        Expression::ArrayLiteral(array) => {
                            let pattern = array.to_pattern(strict)?.into();
                            bindings.push(ArrayPatternElement::PatternRest { pattern });
                        }
                        Expression::ObjectLiteral(object) => {
                            let pattern = object.to_pattern(strict)?.into();
                            bindings.push(ArrayPatternElement::PatternRest { pattern });
                        }
                        _ => return None,
                    }
                    if i + 1 != self.arr.len() {
                        return None;
                    }
                }
                Expression::Assign(assign) => match assign.lhs() {
                    AssignTarget::Identifier(ident) => {
                        bindings.push(ArrayPatternElement::SingleName {
                            ident: *ident,
                            default_init: Some(assign.rhs().clone()),
                        });
                    }
                    AssignTarget::Access(access) => {
                        bindings.push(ArrayPatternElement::PropertyAccess {
                            access: access.clone(),
                        });
                    }
                    AssignTarget::Pattern(pattern) => match pattern {
                        Pattern::Object(pattern) => {
                            bindings.push(ArrayPatternElement::Pattern {
                                pattern: Pattern::Object(pattern.clone()),
                                default_init: Some(assign.rhs().clone()),
                            });
                        }
                        Pattern::Array(pattern) => {
                            bindings.push(ArrayPatternElement::Pattern {
                                pattern: Pattern::Array(pattern.clone()),
                                default_init: Some(assign.rhs().clone()),
                            });
                        }
                    },
                },
                Expression::ArrayLiteral(array) => {
                    let pattern = array.to_pattern(strict)?.into();
                    bindings.push(ArrayPatternElement::Pattern {
                        pattern,
                        default_init: None,
                    });
                }
                Expression::ObjectLiteral(object) => {
                    let pattern = object.to_pattern(strict)?.into();
                    bindings.push(ArrayPatternElement::Pattern {
                        pattern,
                        default_init: None,
                    });
                }
                Expression::PropertyAccess(access) => {
                    bindings.push(ArrayPatternElement::PropertyAccess {
                        access: access.clone(),
                    });
                }
                _ => return None,
            }
        }
        Some(ArrayPattern::new(bindings.into()))
    }
}

impl AsRef<[Option<Expression>]> for ArrayLiteral {
    #[inline]
    fn as_ref(&self) -> &[Option<Expression>] {
        &self.arr
    }
}

impl<T> From<T> for ArrayLiteral
where
    T: Into<Box<[Option<Expression>]>>,
{
    #[inline]
    fn from(decl: T) -> Self {
        Self {
            arr: decl.into(),
            has_trailing_comma_spread: false,
        }
    }
}

impl ToInternedString for ArrayLiteral {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = String::from("[");
        let mut first = true;
        for e in &*self.arr {
            if first {
                first = false;
            } else {
                buf.push_str(", ");
            }
            if let Some(e) = e {
                buf.push_str(&e.to_interned_string(interner));
            }
        }
        buf.push(']');
        buf
    }
}

impl From<ArrayLiteral> for Expression {
    #[inline]
    fn from(arr: ArrayLiteral) -> Self {
        Self::ArrayLiteral(arr)
    }
}

impl VisitWith for ArrayLiteral {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        for expr in self.arr.iter().flatten() {
            try_break!(visitor.visit_expression(expr));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        for expr in self.arr.iter_mut().flatten() {
            try_break!(visitor.visit_expression_mut(expr));
        }
        ControlFlow::Continue(())
    }
}
