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
    pub label: Option<String>,
}

impl ForLoop {
    /// Creates a new for loop AST node.
    pub(in crate::syntax) fn new<I, C, E, B>(
        init: I,
        condition: C,
        final_expr: E,
        body: B,
        label: Option<String>,
    ) -> Self
    where
        I: Into<Option<Node>>,
        C: Into<Option<Node>>,
        E: Into<Option<Node>>,
        B: Into<Node>,
    {
        Self {
            inner: Box::new(InnerForLoop::new(init, condition, final_expr, body)),
            label,
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct WhileLoop {
    cond: Box<Node>,
    expr: Box<Node>,
}

impl WhileLoop {
    pub fn cond(&self) -> &Node {
        &self.cond
    }

    pub fn expr(&self) -> &Node {
        &self.expr
    }

    /// Creates a `WhileLoop` AST node.
    pub fn new<C, B>(condition: C, body: B) -> Self
    where
        C: Into<Node>,
        B: Into<Node>,
    {
        Self {
            cond: Box::new(condition.into()),
            expr: Box::new(body.into()),
        }
    }

    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        write!(f, "while ({}) ", self.cond())?;
        self.expr().display(f, indentation)
    }
}

impl fmt::Display for WhileLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<WhileLoop> for Node {
    fn from(while_loop: WhileLoop) -> Self {
        Self::WhileLoop(while_loop)
    }
}

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct DoWhileLoop {
    body: Box<Node>,
    cond: Box<Node>,
}

impl DoWhileLoop {
    pub fn body(&self) -> &Node {
        &self.body
    }

    pub fn cond(&self) -> &Node {
        &self.cond
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
        }
    }

    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        write!(f, "do")?;
        self.body().display(f, indentation)?;
        write!(f, "while ({})", self.cond())
    }
}

impl fmt::Display for DoWhileLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<DoWhileLoop> for Node {
    fn from(do_while: DoWhileLoop) -> Self {
        Self::DoWhileLoop(do_while)
    }
}

/// The `continue` statement terminates execution of the statements in the current iteration of
/// the current or labeled loop, and continues execution of the loop with the next iteration.
///
/// The continue statement can include an optional label that allows the program to jump to the
/// next iteration of a labeled loop statement instead of the current loop. In this case, the
/// continue statement needs to be nested within this labeled statement.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ContinueStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Continue {
    label: Option<Box<str>>,
}

impl Continue {
    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Box::as_ref)
    }

    /// Creates a `Continue` AST node.
    pub fn new<OL, L>(label: OL) -> Self
    where
        L: Into<Box<str>>,
        OL: Into<Option<L>>,
    {
        Self {
            label: label.into().map(L::into),
        }
    }
}

impl fmt::Display for Continue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "continue{}",
            if let Some(label) = self.label() {
                format!(" {}", label)
            } else {
                String::new()
            }
        )
    }
}

impl From<Continue> for Node {
    fn from(cont: Continue) -> Node {
        Self::Continue(cont)
    }
}
