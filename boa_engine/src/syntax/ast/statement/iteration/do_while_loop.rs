use crate::syntax::ast::{expression::Expression, statement::Statement, ContainsSymbol};
use boa_interner::{Interner, Sym, ToInternedString};

/// The `do...while` statement creates a loop that executes a specified statement until the
/// test condition evaluates to false.
///
/// The condition is evaluated after executing the statement, resulting in the specified
/// statement executing at least once.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-do-while-statement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct DoWhileLoop {
    body: Box<Statement>,
    condition: Expression,
    label: Option<Sym>,
}

impl DoWhileLoop {
    pub fn body(&self) -> &Statement {
        &self.body
    }

    pub fn cond(&self) -> &Expression {
        &self.condition
    }

    pub fn label(&self) -> Option<Sym> {
        self.label
    }

    pub fn set_label(&mut self, label: Sym) {
        self.label = Some(label);
    }

    /// Creates a `DoWhileLoop` AST node.
    pub fn new(body: Statement, condition: Expression) -> Self {
        Self {
            body: body.into(),
            condition,
            label: None,
        }
    }

    /// Converts the "do while" loop to a string with the given indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = if let Some(label) = self.label {
            format!("{}: ", interner.resolve_expect(label))
        } else {
            String::new()
        };
        buf.push_str(&format!(
            "do {} while ({})",
            self.body().to_indented_string(interner, indentation),
            self.cond().to_interned_string(interner)
        ));

        buf
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.body.contains_arguments()
            || self.condition.contains_arguments()
            || matches!(self.label, Some(label) if label == Sym::ARGUMENTS)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.body.contains(symbol) || self.condition.contains(symbol)
    }
}

impl ToInternedString for DoWhileLoop {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<DoWhileLoop> for Statement {
    fn from(do_while: DoWhileLoop) -> Self {
        Self::DoWhileLoop(do_while)
    }
}
