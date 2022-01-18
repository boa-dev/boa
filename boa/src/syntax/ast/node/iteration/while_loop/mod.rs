use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct WhileLoop {
    cond: Box<Node>,
    body: Box<Node>,
    label: Option<Sym>,
}

impl WhileLoop {
    pub fn cond(&self) -> &Node {
        &self.cond
    }

    pub fn body(&self) -> &Node {
        &self.body
    }

    pub fn label(&self) -> Option<Sym> {
        self.label
    }

    pub fn set_label(&mut self, label: Sym) {
        self.label = Some(label);
    }

    /// Creates a `WhileLoop` AST node.
    pub fn new<C, B>(condition: C, body: B) -> Self
    where
        C: Into<Node>,
        B: Into<Node>,
    {
        Self {
            cond: Box::new(condition.into()),
            body: Box::new(body.into()),
            label: None,
        }
    }

    /// Converts the while loop to a string with the given indentation.
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
            "while ({}) {}",
            self.cond().to_interned_string(interner),
            self.body().to_indented_string(interner, indentation)
        ));

        buf
    }
}

impl ToInternedString for WhileLoop {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<WhileLoop> for Node {
    fn from(while_loop: WhileLoop) -> Self {
        Self::WhileLoop(while_loop)
    }
}
