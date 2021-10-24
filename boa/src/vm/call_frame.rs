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
    pub(crate) exit_on_return: bool,
    pub(crate) this: JsValue,
    pub(crate) environment: Environment,
}
