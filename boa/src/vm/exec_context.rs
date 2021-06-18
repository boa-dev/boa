//! This module implements an Execution Context.
//!
//! The execution context is used to track the runtime evalutation of code.
//! Each function's invocation is an execution context, each context has its own "owned" stack.
//!
//! More information:
//! - [ECMAScript reference][spec]
//! [spec]: https://tc39.es/ecma262/#sec-execution-contexts

use crate::Value;

pub struct ExecContext {
    /// The stack will hold values as the execution context runs
    /// As these values are only temporarily held onto the stack and not accessed
    /// anywhere else they don't need to be GC'd
    pub stack: Vec<Value>,
    /// Points to where in the instructions this context should start executing
    /// As the instructions already exist on another stack this only needs to reference the index.
    pub inst_pc: usize,
}

impl Default for ExecContext {
    fn default() -> Self {
        ExecContext {
            stack: vec![],
            inst_pc: 0,
        }
    }
}
