//! Unary expression nodes.
//!
//! A Binary expression comprises any operation applied to a single expression. Some examples include:
//!
//! - [Increment and decrement operations][inc] (`++`, `--`).
//! - The [`delete`][del] operator.
//! - The [bitwise NOT][not] operator (`~`).
//!
//! The full list of valid unary operators is defined in [`UnaryOp`].
//!
//! [inc]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#increment_and_decrement
//! [del]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/delete
//! [not]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_NOT
mod op;

pub use op::*;
use std::ops::ControlFlow;

use boa_interner::{Interner, ToInternedString};

use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::syntax::ast::{expression::Expression, ContainsSymbol};

/// A unary expression is an operation with only one operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary_operators
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Unary {
    op: UnaryOp,
    target: Box<Expression>,
}

impl Unary {
    /// Creates a new `UnaryOp` AST Expression.
    pub(in crate::syntax) fn new(op: UnaryOp, target: Expression) -> Self {
        Self {
            op,
            target: Box::new(target),
        }
    }

    /// Gets the unary operation of the Expression.
    #[inline]
    pub fn op(&self) -> UnaryOp {
        self.op
    }

    /// Gets the target of this unary operator.
    #[inline]
    pub fn target(&self) -> &Expression {
        self.target.as_ref()
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.target.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.target.contains(symbol)
    }
}

impl ToInternedString for Unary {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        let space = match self.op {
            UnaryOp::TypeOf | UnaryOp::Delete | UnaryOp::Void => " ",
            _ => "",
        };
        format!(
            "{}{space}{}",
            self.op,
            self.target.to_interned_string(interner)
        )
    }
}

impl From<Unary> for Expression {
    #[inline]
    fn from(op: Unary) -> Self {
        Self::Unary(op)
    }
}

impl VisitWith for Unary {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_expression(&*self.target)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_expression_mut(&mut *self.target)
    }
}
