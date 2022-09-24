use crate::syntax::ast::expression::Expression;
use boa_interner::{Interner, ToInternedString};

/// The `conditional` (ternary) operation is the only JavaScript operation that takes three
/// operands.
///
/// This operation takes three operands: a condition followed by a question mark (`?`),
/// then an expression to execute `if` the condition is truthy followed by a colon (`:`),
/// and finally the expression to execute if the condition is `false`.
/// This operator is frequently used as a shortcut for the `if` statement.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ConditionalExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Literals
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Conditional {
    condition: Box<Expression>,
    if_true: Box<Expression>,
    if_false: Box<Expression>,
}

impl Conditional {
    pub fn cond(&self) -> &Expression {
        &self.condition
    }

    pub fn if_true(&self) -> &Expression {
        &self.if_true
    }

    pub fn if_false(&self) -> &Expression {
        &self.if_false
    }

    /// Creates a `ConditionalOp` AST Expression.
    pub fn new(condition: Expression, if_true: Expression, if_false: Expression) -> Self {
        Self {
            condition: Box::new(condition),
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
        }
    }
}

impl ToInternedString for Conditional {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{} ? {} : {}",
            self.cond().to_interned_string(interner),
            self.if_true().to_interned_string(interner),
            self.if_false().to_interned_string(interner)
        )
    }
}

impl From<Conditional> for Expression {
    fn from(cond_op: Conditional) -> Self {
        Self::Conditional(cond_op)
    }
}
