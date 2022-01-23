//! CallFrame
//! This module will provides everything needed to implement the CallFrame

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
    pub(crate) pop_env_on_return: usize,
    pub(crate) param_count: usize,
    pub(crate) arg_count: usize,
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
