use crate::syntax::ast::{expression::Expression, statement::Statement, ContainsSymbol};
use boa_interner::{Interner, Sym, ToInternedString};
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
    label: Option<Sym>,
}

impl WhileLoop {
    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn body(&self) -> &Statement {
        &self.body
    }

    pub fn label(&self) -> Option<Sym> {
        self.label
    }

    pub fn set_label(&mut self, label: Sym) {
        self.label = Some(label);
    }

    /// Creates a `WhileLoop` AST node.
    pub fn new(condition: Expression, body: Statement) -> Self {
        Self {
            condition,
            body: body.into(),
            label: None,
        }
    }

    /// Converts the while loop to a string with the given indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = if let Some(label) = self.label {
            format!("{}: ", interner.resolve_expect(label))
        } else {
            String::new()
        };
        buf.push_str(&format!(
            "while ({}) {}",
            self.condition().to_interned_string(interner),
            self.body().to_indented_string(interner, indentation)
        ));

        buf
    }

    pub(crate) fn contains_arguments(&self) -> bool {
        self.condition.contains_arguments()
            || self.body.contains_arguments()
            || matches!(self.label, Some(label) if label == Sym::ARGUMENTS)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.condition.contains(symbol) || self.body.contains(symbol)
    }
}

impl ToInternedString for WhileLoop {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<WhileLoop> for Statement {
    fn from(while_loop: WhileLoop) -> Self {
        Self::WhileLoop(while_loop)
    }
}
