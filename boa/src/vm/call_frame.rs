//! `CallFrame`
//!
//! This module will provides everything needed to implement the `CallFrame`

use super::CodeBlock;
use crate::{gc::Gc, JsValue};

#[derive(Debug)]
pub struct CallFrame {
    pub(crate) prev: Option<Box<Self>>,
    pub(crate) code: Gc<CodeBlock>,
    pub(crate) pc: usize,
    pub(crate) this: JsValue,
    pub(crate) catch: Vec<CatchAddresses>,
    pub(crate) finally_return: FinallyReturn,
    pub(crate) finally_jump: Vec<Option<u32>>,
    pub(crate) pop_on_return: usize,

    // Tracks the number of environments in the current loop block.
    // On abrupt returns this is used to decide how many environments need to be pop'ed.
    pub(crate) loop_env_stack: Vec<usize>,

    // Tracks the number of environments in the current try-catch-finally block.
    // On abrupt returns this is used to decide how many environments need to be pop'ed.
    pub(crate) try_env_stack: Vec<TryStackEntry>,

    pub(crate) param_count: usize,
    pub(crate) arg_count: usize,
}

impl CallFrame {
    /// Tracks that one environment has been pushed in the current loop block.
    pub(crate) fn loop_env_stack_inc(&mut self) {
        *self
            .loop_env_stack
            .last_mut()
            .expect("loop environment stack entry must exist") += 1;
    }

    /// Tracks that one environment has been pop'ed in the current loop block.
    pub(crate) fn loop_env_stack_dec(&mut self) {
        *self
            .loop_env_stack
            .last_mut()
            .expect("loop environment stack entry must exist") -= 1;
    }

    /// Tracks that one environment has been pushed in the current try-catch-finally block.
    pub(crate) fn try_env_stack_inc(&mut self) {
        self.try_env_stack
            .last_mut()
            .expect("try environment stack entry must exist")
            .num_env += 1;
    }

    /// Tracks that one environment has been pop'ed in the current try-catch-finally block.
    pub(crate) fn try_env_stack_dec(&mut self) {
        self.try_env_stack
            .last_mut()
            .expect("try environment stack entry must exist")
            .num_env -= 1;
    }

    /// Tracks that one loop has started in the current try-catch-finally block.
    pub(crate) fn try_env_stack_loop_inc(&mut self) {
        self.try_env_stack
            .last_mut()
            .expect("try environment stack entry must exist")
            .num_loop_stack_entries += 1;
    }

    /// Tracks that one loop has finished in the current try-catch-finally block.
    pub(crate) fn try_env_stack_loop_dec(&mut self) {
        self.try_env_stack
            .last_mut()
            .expect("try environment stack entry must exist")
            .num_loop_stack_entries -= 1;
    }
}

/// Tracks the number of environments in the current try-catch-finally block.
///
/// Because of the interactions between loops and try-catch-finally blocks,
/// the number of loop blocks in the try-catch-finally block also needs to be tracked.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TryStackEntry {
    pub(crate) num_env: usize,
    pub(crate) num_loop_stack_entries: usize,
}

#[derive(Debug)]
pub(crate) struct CatchAddresses {
    pub(crate) next: u32,
    pub(crate) finally: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum FinallyReturn {
    None,
    Ok,
    Err,
}
