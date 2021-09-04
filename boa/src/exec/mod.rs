//! Execution of the AST, this is where the interpreter actually runs

#[cfg(test)]
mod tests;

use crate::{Context, JsResult, JsValue};

pub trait Executable {
    /// Runs this executable in the given context.
    fn run(&self, context: &mut Context) -> JsResult<JsValue>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum InterpreterState {
    Executing,
    Return,
    Break(Option<Box<str>>),
    Continue(Option<Box<str>>),
}

/// A Javascript intepreter
#[derive(Debug)]
pub struct Interpreter {
    /// the current state of the interpreter.
    state: InterpreterState,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Creates a new interpreter.
    pub fn new() -> Self {
        Self {
            state: InterpreterState::Executing,
        }
    }

    #[inline]
    pub(crate) fn set_current_state(&mut self, new_state: InterpreterState) {
        self.state = new_state
    }

    #[inline]
    pub(crate) fn get_current_state(&self) -> &InterpreterState {
        &self.state
    }
}
