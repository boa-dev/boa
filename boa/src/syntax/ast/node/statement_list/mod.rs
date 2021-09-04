//! Statement list node.

use crate::{
    context::StrictType,
    exec::{Executable, InterpreterState},
    gc::{empty_trace, Finalize, Trace},
    syntax::ast::node::{Declaration, Node},
    BoaProfiler, Context, JsResult, JsValue,
};
use std::{collections::HashSet, fmt, ops::Deref, rc::Rc};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

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
    strict: bool,
}

impl StatementList {
    /// Gets the list of items.
    #[inline]
    pub fn items(&self) -> &[Node] {
        &self.items
    }

    /// Get the strict mode.
    #[inline]
    pub fn strict(&self) -> bool {
        self.strict
    }

    /// Set the strict mode.
    #[inline]
    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        // Print statements
        for node in self.items.iter() {
            // We rely on the node to add the correct indent.
            node.display(f, indentation)?;

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
                    // It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains any duplicate entries.
                    // https://tc39.es/ecma262/#sec-block-static-semantics-early-errors
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            if !set.insert(ident.as_ref()) {
                                unreachable!("Redeclaration of {}", ident.as_ref());
                            }
                        }
                        Declaration::Pattern(p) => {
                            for ident in p.idents() {
                                if !set.insert(ident) {
                                    unreachable!("Redeclaration of {}", ident);
                                }
                            }
                        }
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
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            set.insert(ident.as_ref());
                        }
                        Declaration::Pattern(p) => {
                            for ident in p.idents() {
                                set.insert(ident.as_ref());
                            }
                        }
                    }
                }
            }
        }
        set
    }
}

impl Executable for StatementList {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("StatementList", "exec");

        // https://tc39.es/ecma262/#sec-block-runtime-semantics-evaluation
        // The return value is uninitialized, which means it defaults to Value::Undefined
        let mut obj = JsValue::default();
        context
            .executor()
            .set_current_state(InterpreterState::Executing);

        let strict_before = context.strict_type();

        match context.strict_type() {
            StrictType::Off if self.strict => context.set_strict(StrictType::Function),
            StrictType::Function if !self.strict => context.set_strict_mode_off(),
            _ => {}
        }

        for (i, item) in self.items().iter().enumerate() {
            let val = match item.run(context) {
                Ok(val) => val,
                Err(e) => {
                    context.set_strict(strict_before);
                    return Err(e);
                }
            };
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

        context.set_strict(strict_before);

        Ok(obj)
    }
}

impl<T> From<T> for StatementList
where
    T: Into<Box<[Node]>>,
{
    fn from(stm: T) -> Self {
        Self {
            items: stm.into(),
            strict: false,
        }
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
