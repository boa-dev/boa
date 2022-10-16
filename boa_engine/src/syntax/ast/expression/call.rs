use crate::syntax::ast::{join_nodes, ContainsSymbol};
use boa_interner::{Interner, ToInternedString};

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
    target: Box<Expression>,
    args: Box<[Expression]>,
}

impl Call {
    /// Creates a new `Call` AST Expression.
    #[inline]
    pub fn new(target: Expression, args: Box<[Expression]>) -> Self {
        Self {
            target: target.into(),
            args,
        }
    }

    /// Gets the name of the function call.
    #[inline]
    pub fn expr(&self) -> &Expression {
        &self.target
    }

    /// Retrieves the arguments passed to the function.
    #[inline]
    pub fn args(&self) -> &[Expression] {
        &self.args
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.target.contains_arguments() || self.args.iter().any(Expression::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.target.contains(symbol) || self.args.iter().any(|expr| expr.contains(symbol))
    }
}

impl ToInternedString for Call {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{}({})",
            self.target.to_interned_string(interner),
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
