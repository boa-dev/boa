use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::syntax::ast::{join_nodes, ContainsSymbol};
use crate::try_break;
use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

use super::Expression;

/// Calling the function actually performs the specified actions with the indicated parameters.
///
/// Defining a function does not execute it. Defining it simply names the function and
/// specifies what to do when the function is called. Functions must be in scope when they are
/// called, but the function declaration can be hoisted. The scope of a function is the
/// function in which it is declared (or the entire program, if it is declared at the top
/// level).
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-CallExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Functions#Calling_functions
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    function: Box<Expression>,
    args: Box<[Expression]>,
}

impl Call {
    /// Creates a new `Call` AST Expression.
    #[inline]
    pub fn new(function: Expression, args: Box<[Expression]>) -> Self {
        Self {
            function: function.into(),
            args,
        }
    }

    /// Gets the target function of this call expression.
    #[inline]
    pub fn function(&self) -> &Expression {
        &self.function
    }

    /// Retrieves the arguments passed to the function.
    #[inline]
    pub fn args(&self) -> &[Expression] {
        &self.args
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.function.contains_arguments() || self.args.iter().any(Expression::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.function.contains(symbol) || self.args.iter().any(|expr| expr.contains(symbol))
    }
}

impl ToInternedString for Call {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{}({})",
            self.function.to_interned_string(interner),
            join_nodes(interner, &self.args)
        )
    }
}

impl From<Call> for Expression {
    #[inline]
    fn from(call: Call) -> Self {
        Self::Call(call)
    }
}

impl VisitWith for Call {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_expression(&self.function));
        for expr in self.args.iter() {
            try_break!(visitor.visit_expression(expr));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_expression_mut(&mut self.function));
        for expr in self.args.iter_mut() {
            try_break!(visitor.visit_expression_mut(expr));
        }
        ControlFlow::Continue(())
    }
}

/// The `super` keyword is used to access and call functions on an object's parent.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-SuperCall
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/super
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SuperCall {
    args: Box<[Expression]>,
}

impl SuperCall {
    /// Creates a new `SuperCall` AST node.
    pub(crate) fn new<A>(args: A) -> Self
    where
        A: Into<Box<[Expression]>>,
    {
        Self { args: args.into() }
    }

    /// Retrieves the arguments of the super call.
    pub(crate) fn args(&self) -> &[Expression] {
        &self.args
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.args.iter().any(Expression::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.args.iter().any(|expr| expr.contains(symbol))
    }
}

impl ToInternedString for SuperCall {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("super({})", join_nodes(interner, &self.args))
    }
}

impl From<SuperCall> for Expression {
    #[inline]
    fn from(call: SuperCall) -> Self {
        Self::SuperCall(call)
    }
}

impl VisitWith for SuperCall {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        for expr in self.args.iter() {
            try_break!(visitor.visit_expression(expr));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        for expr in self.args.iter_mut() {
            try_break!(visitor.visit_expression_mut(expr));
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fmt() {
        crate::syntax::ast::test_formatting(
            r#"
        call_1(1, 2, 3);
        call_2("argument here");
        call_3();
        "#,
        );
    }
}
