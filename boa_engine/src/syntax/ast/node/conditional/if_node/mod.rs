use crate::syntax::ast::node::Node;
use boa_gc::{Finalize, Trace};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The `if` statement executes a statement if a specified condition is [`truthy`][truthy]. If
/// the condition is [`falsy`][falsy], another statement can be executed.
///
/// Multiple `if...else` statements can be nested to create an else if clause.
///
/// Note that there is no elseif (in one word) keyword in JavaScript.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-IfStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
/// [truthy]: https://developer.mozilla.org/en-US/docs/Glossary/truthy
/// [falsy]: https://developer.mozilla.org/en-US/docs/Glossary/falsy
/// [expression]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Expressions
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct If {
    cond: Box<Node>,
    body: Box<Node>,
    else_node: Option<Box<Node>>,
}

impl If {
    pub fn cond(&self) -> &Node {
        &self.cond
    }

    pub fn body(&self) -> &Node {
        &self.body
    }

    pub fn else_node(&self) -> Option<&Node> {
        self.else_node.as_ref().map(Box::as_ref)
    }

    /// Creates an `If` AST node.
    pub fn new<C, B, E, OE>(condition: C, body: B, else_node: OE) -> Self
    where
        C: Into<Node>,
        B: Into<Node>,
        E: Into<Node>,
        OE: Into<Option<E>>,
    {
        Self {
            cond: Box::new(condition.into()),
            body: Box::new(body.into()),
            else_node: else_node.into().map(E::into).map(Box::new),
        }
    }

    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indent: usize,
    ) -> String {
        let mut buf = format!("if ({}) ", self.cond().to_interned_string(interner));
        match self.else_node() {
            Some(else_e) => {
                buf.push_str(&format!(
                    "{} else {}",
                    self.body().to_indented_string(interner, indent),
                    else_e.to_indented_string(interner, indent)
                ));
            }
            None => {
                buf.push_str(&self.body().to_indented_string(interner, indent));
            }
        }
        buf
    }
}

impl ToInternedString for If {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<If> for Node {
    fn from(if_stm: If) -> Self {
        Self::If(if_stm)
    }
}
