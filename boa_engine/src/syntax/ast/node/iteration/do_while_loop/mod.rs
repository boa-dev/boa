use crate::syntax::ast::node::Node;
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct DoWhileLoop {
    body: Box<Node>,
    cond: Box<Node>,
    label: Option<Sym>,
}

impl DoWhileLoop {
    pub fn body(&self) -> &Node {
        &self.body
    }

    pub fn cond(&self) -> &Node {
        &self.cond
    }

    pub fn label(&self) -> Option<Sym> {
        self.label
    }

    pub fn set_label(&mut self, label: Sym) {
        self.label = Some(label);
    }

    /// Creates a `DoWhileLoop` AST node.
    pub fn new<B, C>(body: B, condition: C) -> Self
    where
        B: Into<Node>,
        C: Into<Node>,
    {
        Self {
            body: Box::new(body.into()),
            cond: Box::new(condition.into()),
            label: None,
        }
    }

    /// Converts the "do while" loop to a string with the given indentation.
    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indentation: usize,
    ) -> String {
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
}

impl ToInternedString for DoWhileLoop {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<DoWhileLoop> for Node {
    fn from(do_while: DoWhileLoop) -> Self {
        Self::DoWhileLoop(do_while)
    }
}
