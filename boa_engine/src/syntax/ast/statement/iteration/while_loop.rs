use crate::syntax::ast::{expression::Expression, statement::Statement, ContainsSymbol};
use boa_interner::{Interner, ToIndentedString, ToInternedString};

/// The `while` statement creates a loop that executes a specified statement as long as the
/// test condition evaluates to `true`.
///
/// The condition is evaluated before executing the statement.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-grammar-notation-WhileStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/while
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct WhileLoop {
    condition: Expression,
    body: Box<Statement>,
}

impl WhileLoop {
    /// Creates a `WhileLoop` AST node.
    #[inline]
    pub fn new(condition: Expression, body: Statement) -> Self {
        Self {
            condition,
            body: body.into(),
        }
    }

    /// Gets the condition of the while loop.
    #[inline]
    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    /// Gets the body of the while loop.
    #[inline]
    pub fn body(&self) -> &Statement {
        &self.body
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.condition.contains_arguments() || self.body.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.condition.contains(symbol) || self.body.contains(symbol)
    }
}

impl ToIndentedString for WhileLoop {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "while ({}) {}",
            self.condition().to_interned_string(interner),
            self.body().to_indented_string(interner, indentation)
        )
    }
}

impl From<WhileLoop> for Statement {
    #[inline]
    fn from(while_loop: WhileLoop) -> Self {
        Self::WhileLoop(while_loop)
    }
}
