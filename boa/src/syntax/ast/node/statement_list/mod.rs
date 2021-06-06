//! Statement list node.

use crate::{
    exec::{Executable, InterpreterState},
    gc::{empty_trace, Finalize, Trace},
    syntax::ast::node::Node,
    BoaProfiler, Context, Result, Value,
};
use std::{collections::HashSet, fmt, ops::Deref, rc::Rc};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "vm")]
use crate::vm::{compilation::CodeGen, Compiler};

/// List of statements.
///
/// Similar to `Node::Block` but without the braces.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-StatementList
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct StatementList {
    #[cfg_attr(feature = "deser", serde(flatten))]
    items: Box<[Node]>,
}

impl StatementList {
    /// Gets the list of items.
    pub fn items(&self) -> &[Node] {
        &self.items
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        let indent = "    ".repeat(indentation);
        // Print statements
        for node in self.items.iter() {
            f.write_str(&indent)?;
            node.display(f, indentation + 1)?;

            match node {
                Node::Block(_) | Node::If(_) | Node::Switch(_) | Node::WhileLoop(_) => {}
                _ => write!(f, ";")?,
            }
            writeln!(f)?;
        }
        Ok(())
    }

    pub fn lexically_declared_names(&self) -> HashSet<&str> {
        let mut set = HashSet::new();
        for stmt in self.items() {
            if let Node::LetDeclList(decl_list) | Node::ConstDeclList(decl_list) = stmt {
                for decl in decl_list.as_ref() {
                    if !set.insert(decl.name()) {
                        // It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains any duplicate entries.
                        // https://tc39.es/ecma262/#sec-block-static-semantics-early-errors
                        unreachable!("Redeclaration of {}", decl.name());
                    }
                }
            }
        }
        set
    }

    pub fn function_declared_names(&self) -> HashSet<&str> {
        let mut set = HashSet::new();
        for stmt in self.items() {
            if let Node::FunctionDecl(decl) = stmt {
                set.insert(decl.name());
            }
        }
        set
    }

    pub fn var_declared_names(&self) -> HashSet<&str> {
        let mut set = HashSet::new();
        for stmt in self.items() {
            if let Node::VarDeclList(decl_list) = stmt {
                for decl in decl_list.as_ref() {
                    set.insert(decl.name());
                }
            }
        }
        set
    }
}

impl Executable for StatementList {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("StatementList", "exec");

        // https://tc39.es/ecma262/#sec-block-runtime-semantics-evaluation
        // The return value is uninitialized, which means it defaults to Value::Undefined
        let mut obj = Value::default();
        context
            .executor()
            .set_current_state(InterpreterState::Executing);
        for (i, item) in self.items().iter().enumerate() {
            let val = item.run(context)?;
            match context.executor().get_current_state() {
                InterpreterState::Return => {
                    // Early return.
                    obj = val;
                    break;
                }
                InterpreterState::Break(_label) => {
                    // Early break.
                    break;
                }
                InterpreterState::Continue(_label) => {
                    break;
                }
                InterpreterState::Executing => {
                    // Continue execution
                }
            }
            if i + 1 == self.items().len() {
                obj = val;
            }
        }

        Ok(obj)
    }
}

#[cfg(feature = "vm")]
impl CodeGen for StatementList {
    fn compile(&self, compiler: &mut Compiler) {
        let _timer = BoaProfiler::global().start_event("StatementList - Code Gen", "codeGen");

        for item in self.items().iter() {
            item.compile(compiler);
        }
    }
}

impl<T> From<T> for StatementList
where
    T: Into<Box<[Node]>>,
{
    fn from(stm: T) -> Self {
        Self { items: stm.into() }
    }
}

impl fmt::Display for StatementList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

// List of statements wrapped with Rc. We need this for self mutating functions.
// Since we need to cheaply clone the function body and drop the borrow of the function object to
// mutably borrow the function object and call this cloned function body
#[derive(Clone, Debug, Finalize, PartialEq)]
pub struct RcStatementList(Rc<StatementList>);

impl Deref for RcStatementList {
    type Target = StatementList;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<StatementList> for RcStatementList {
    #[inline]
    fn from(statementlist: StatementList) -> Self {
        Self(Rc::from(statementlist))
    }
}

// SAFETY: This is safe for types not containing any `Trace` types.
unsafe impl Trace for RcStatementList {
    empty_trace!();
}
