//! Iteration nodes

pub use self::{
    continue_node::Continue, do_while_loop::DoWhileLoop, for_in_loop::ForInLoop, for_loop::ForLoop,
    for_of_loop::ForOfLoop, while_loop::WhileLoop,
};
use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::{declaration::Declaration, identifier::Identifier},
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum IterableLoopInitializer {
    Identifier(Identifier),
    Var(Declaration),
    Let(Declaration),
    Const(Declaration),
}

impl fmt::Display for IterableLoopInitializer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IterableLoopInitializer::Identifier(identifier) => write!(f, "{}", identifier),
            IterableLoopInitializer::Var(declaration) => write!(f, "var {}", declaration),
            IterableLoopInitializer::Let(declaration) => write!(f, "let {}", declaration),
            IterableLoopInitializer::Const(declaration) => write!(f, "const {}", declaration),
        }
    }
}

pub mod continue_node;
pub mod do_while_loop;
pub mod for_in_loop;
pub mod for_loop;
pub mod for_of_loop;
pub mod while_loop;
