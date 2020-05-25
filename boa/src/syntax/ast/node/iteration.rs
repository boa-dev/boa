use super::Node;
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `for` statement creates a loop that consists of three optional expressions.
///
/// A `for` loop repeats until a specified condition evaluates to `false`.
/// The JavaScript for loop is similar to the Java and C for loop.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ForDeclaration
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ForLoop {
    #[cfg_attr(feature = "serde", serde(flatten))]
    inner: Box<InnerForLoop>,
}

impl ForLoop {
    /// Creates a new for loop AST node.
    pub(in crate::syntax) fn new<I, C, E, B>(init: I, condition: C, final_expr: E, body: B) -> Self
    where
        I: Into<Option<Node>>,
        C: Into<Option<Node>>,
        E: Into<Option<Node>>,
        B: Into<Node>,
    {
        Self {
            inner: Box::new(InnerForLoop::new(init, condition, final_expr, body)),
        }
    }

    /// Gets the initialization node.
    pub fn init(&self) -> Option<&Node> {
        self.inner.init()
    }

    /// Gets the loop condition node.
    pub fn condition(&self) -> Option<&Node> {
        self.inner.condition()
    }

    /// Gets the final expression node.
    pub fn final_expr(&self) -> Option<&Node> {
        self.inner.final_expr()
    }

    /// Gets the body of the for loop.
    pub fn body(&self) -> &Node {
        self.inner.body()
    }

    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        f.write_str("for (")?;
        if let Some(init) = self.init() {
            fmt::Display::fmt(init, f)?;
        }
        f.write_str(";")?;
        if let Some(condition) = self.condition() {
            fmt::Display::fmt(condition, f)?;
        }
        f.write_str(";")?;
        if let Some(final_expr) = self.final_expr() {
            fmt::Display::fmt(final_expr, f)?;
        }
        writeln!(f, ") {{")?;

        self.inner.body().display(f, indentation + 1)?;

        write!(f, "}}")
    }
}

impl fmt::Display for ForLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<ForLoop> for Node {
    fn from(for_loop: ForLoop) -> Self {
        Self::ForLoop(for_loop)
    }
}

/// Inner structure to avoid multiple indirections in the heap.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
struct InnerForLoop {
    init: Option<Node>,
    condition: Option<Node>,
    final_expr: Option<Node>,
    body: Node,
}

impl InnerForLoop {
    /// Creates a new inner for loop.
    fn new<I, C, E, B>(init: I, condition: C, final_expr: E, body: B) -> Self
    where
        I: Into<Option<Node>>,
        C: Into<Option<Node>>,
        E: Into<Option<Node>>,
        B: Into<Node>,
    {
        Self {
            init: init.into(),
            condition: condition.into(),
            final_expr: final_expr.into(),
            body: body.into(),
        }
    }

    /// Gets the initialization node.
    fn init(&self) -> Option<&Node> {
        self.init.as_ref()
    }

    /// Gets the loop condition node.
    fn condition(&self) -> Option<&Node> {
        self.condition.as_ref()
    }

    /// Gets the final expression node.
    fn final_expr(&self) -> Option<&Node> {
        self.final_expr.as_ref()
    }

    /// Gets the body of the for loop.
    fn body(&self) -> &Node {
        &self.body
    }
}
