use crate::{
    environment::declarative_environment_record::DeclarativeEnvironmentRecord,
    exec::{Executable, InterpreterState},
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
    BoaProfiler, Context, JsResult, JsValue,
};
use std::fmt;

#[cfg(feature = "deser")]
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ForLoop {
    #[cfg_attr(feature = "deser", serde(flatten))]
    inner: Box<InnerForLoop>,
    label: Option<Box<str>>,
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
            label: None,
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

    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        if let Some(ref label) = self.label {
            write!(f, "{}: ", label)?;
        }
        f.write_str("for (")?;
        if let Some(init) = self.init() {
            fmt::Display::fmt(init, f)?;
        }
        f.write_str("; ")?;
        if let Some(condition) = self.condition() {
            fmt::Display::fmt(condition, f)?;
        }
        f.write_str("; ")?;
        if let Some(final_expr) = self.final_expr() {
            fmt::Display::fmt(final_expr, f)?;
        }
        write!(f, ") ")?;
        self.inner.body().display(f, indentation)
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Box::as_ref)
    }

    pub fn set_label(&mut self, label: Box<str>) {
        self.label = Some(label);
    }
}

impl Executable for ForLoop {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        // Create the block environment.
        let _timer = BoaProfiler::global().start_event("ForLoop", "exec");
        {
            let env = context.get_current_environment();
            context.push_environment(DeclarativeEnvironmentRecord::new(Some(env)));
        }

        if let Some(init) = self.init() {
            init.run(context)?;
        }

        while self
            .condition()
            .map(|cond| cond.run(context).map(|v| v.to_boolean()))
            .transpose()?
            .unwrap_or(true)
        {
            let result = self.body().run(context)?;

            match context.executor().get_current_state() {
                InterpreterState::Break(label) => {
                    handle_state_with_labels!(self, label, context, break);
                    break;
                }
                InterpreterState::Continue(label) => {
                    handle_state_with_labels!(self, label, context, continue);
                }

                InterpreterState::Return => {
                    return Ok(result);
                }
                InterpreterState::Executing => {
                    // Continue execution.
                }
            }

            if let Some(final_expr) = self.final_expr() {
                final_expr.run(context)?;
            }
        }

        // pop the block env
        let _ = context.pop_environment();

        Ok(JsValue::undefined())
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
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
