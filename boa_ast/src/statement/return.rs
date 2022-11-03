use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{expression::Expression, statement::Statement, ContainsSymbol};
use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

/// The `return` statement ends function execution and specifies a value to be returned to the
/// function caller.
///
/// Syntax: `return [expression];`
///
/// `expression`:
///  > The expression whose value is to be returned. If omitted, `undefined` is returned instead.
///
/// When a `return` statement is used in a function body, the execution of the function is
/// stopped. If specified, a given value is returned to the function caller.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ReturnStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Return {
    target: Option<Expression>,
}

impl Return {
    /// Gets the target expression value of this `Return` statement.
    #[must_use]
    pub fn target(&self) -> Option<&Expression> {
        self.target.as_ref()
    }

    /// Creates a `Return` AST node.
    #[must_use]
    pub fn new(expression: Option<Expression>) -> Self {
        Self { target: expression }
    }

    pub(crate) fn contains_arguments(&self) -> bool {
        matches!(self.target, Some(ref expr) if expr.contains_arguments())
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        matches!(self.target, Some(ref expr) if expr.contains(symbol))
    }
}

impl From<Return> for Statement {
    fn from(return_smt: Return) -> Self {
        Self::Return(return_smt)
    }
}

impl ToInternedString for Return {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self.target() {
            Some(ex) => format!("return {}", ex.to_interned_string(interner)),
            None => "return".to_owned(),
        }
    }
}

impl VisitWith for Return {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(expr) = &self.target {
            visitor.visit_expression(expr)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(expr) = &mut self.target {
            visitor.visit_expression_mut(expr)
        } else {
            ControlFlow::Continue(())
        }
    }
}
