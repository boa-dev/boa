//! CallFrame
//! This module will provides everything needed to implement the CallFrame

use super::CodeBlock;
use crate::{environment::lexical_environment::Environment, JsValue};
use gc::Gc;

#[derive(Debug)]
pub struct CallFrame {
    pub(crate) prev: Option<Box<Self>>,
    pub(crate) code: Gc<CodeBlock>,
    pub(crate) pc: usize,
    pub(crate) fp: usize,
    pub(crate) this: JsValue,
    pub(crate) environment: Environment,
    pub(crate) catch: Option<u32>,
    pub(crate) pop_env_on_return: usize,
    pub(crate) finally_no_jump: bool,
    pub(crate) param_count: usize,
    pub(crate) arg_count: usize,
}
