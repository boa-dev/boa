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
    pub(crate) loop_env_stack: Vec<usize>,
    pub(crate) try_env_stack: Vec<TryStackEntry>,
    pub(crate) param_count: usize,
    pub(crate) arg_count: usize,
}

impl CallFrame {
    pub(crate) fn loop_env_stack_inc(&mut self) {
        *self
            .loop_env_stack
            .last_mut()
            .expect("loop environment stack entry must exist") += 1;
    }

    pub(crate) fn loop_env_stack_dec(&mut self) {
        *self
            .loop_env_stack
            .last_mut()
            .expect("loop environment stack entry must exist") -= 1;
    }

    pub(crate) fn try_env_stack_inc(&mut self) {
        self.try_env_stack
            .last_mut()
            .expect("try environment stack entry must exist")
            .num_env += 1;
    }

    pub(crate) fn try_env_stack_dec(&mut self) {
        self.try_env_stack
            .last_mut()
            .expect("try environment stack entry must exist")
            .num_env -= 1;
    }

    pub(crate) fn try_env_stack_loop_inc(&mut self) {
        self.try_env_stack
            .last_mut()
            .expect("try environment stack entry must exist")
            .num_loop_stack_entries += 1;
    }

    pub(crate) fn try_env_stack_loop_dec(&mut self) {
        self.try_env_stack
            .last_mut()
            .expect("try environment stack entry must exist")
            .num_loop_stack_entries -= 1;
    }
}

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
