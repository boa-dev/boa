//! `CallFrame`
//!
//! This module will provides everything needed to implement the `CallFrame`

use crate::{object::JsObject, vm::CodeBlock};
use boa_gc::{Finalize, Gc, Trace};

mod abrupt_record;
mod env_stack;

pub(crate) use abrupt_record::AbruptCompletionRecord;
pub(crate) use env_stack::EnvStackEntry;

/// A `CallFrame` holds the state of a function call.
#[derive(Clone, Debug, Finalize, Trace)]
pub struct CallFrame {
    pub(crate) code_block: Gc<CodeBlock>,
    pub(crate) pc: usize,
    #[unsafe_ignore_trace]
    pub(crate) try_catch: Vec<FinallyAddresses>,
    #[unsafe_ignore_trace]
    pub(crate) finally_return: FinallyReturn,
    #[unsafe_ignore_trace]
    pub(crate) abrupt_completion: Option<AbruptCompletionRecord>,
    pub(crate) pop_on_return: usize,
    // Tracks the number of environments in environment entry.
    // On abrupt returns this is used to decide how many environments need to be pop'ed.
    #[unsafe_ignore_trace]
    pub(crate) env_stack: Vec<EnvStackEntry>,
    pub(crate) param_count: usize,
    pub(crate) arg_count: usize,
    #[unsafe_ignore_trace]
    pub(crate) generator_resume_kind: GeneratorResumeKind,

    // Indicate that the last try block has thrown an exception.
    pub(crate) thrown: bool,

    // When an async generator is resumed, the generator object is needed
    // to fulfill the steps 4.e-j in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart).
    pub(crate) async_generator: Option<JsObject>,
}

/// ---- `CallFrame` creation methods ----
impl CallFrame {
    /// Creates a new `CallFrame` with the provided `CodeBlock`.
    pub(crate) fn new(code_block: Gc<CodeBlock>) -> Self {
        let max_length = code_block.bytecode.len() as u32;
        Self {
            code_block,
            pc: 0,
            try_catch: Vec::new(),
            finally_return: FinallyReturn::None,
            pop_on_return: 0,
            env_stack: Vec::from([EnvStackEntry::new(0, max_length)]),
            abrupt_completion: None,
            param_count: 0,
            arg_count: 0,
            generator_resume_kind: GeneratorResumeKind::Normal,
            thrown: false,
            async_generator: None,
        }
    }

    /// Updates a `CallFrame`'s `param_count` field with the value provided.
    pub(crate) fn with_param_count(mut self, count: usize) -> Self {
        self.param_count = count;
        self
    }

    /// Updates a `CallFrame`'s `arg_count` field with the value provided.
    pub(crate) fn with_arg_count(mut self, count: usize) -> Self {
        self.arg_count = count;
        self
    }
}

/// ---- `CallFrame` stack methods ----
impl CallFrame {
    /// Tracks that one environment has been pushed in the current loop block.
    pub(crate) fn inc_frame_env_stack(&mut self) {
        self.env_stack
            .last_mut()
            .expect("environment stack entry must exist")
            .inc_env_num();
    }

    /// Tracks that one environment has been pop'ed in the current loop block.
    ///
    /// Note:
    ///  - This will check if the env stack has reached 0 and should be popped.
    pub(crate) fn dec_frame_env_stack(&mut self) {
        self.env_stack
            .last_mut()
            .expect("environment stack entry must exist")
            .dec_env_num();
    }
}

/// Tracks the address that should be jumped to when an error is caught.
///
/// Additionally the address of a finally block is tracked, to allow for special handling if it exists.
#[derive(Copy, Clone, Debug)]
pub(crate) struct FinallyAddresses {
    finally: Option<u32>,
}

impl FinallyAddresses {
    pub(crate) const fn new(finally_address: Option<u32>) -> Self {
        Self {
            finally: finally_address,
        }
    }

    pub(crate) const fn finally(self) -> Option<u32> {
        self.finally
    }
}

/// Indicates if a function should return or throw at the end of a finally block.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum FinallyReturn {
    None,
    Ok,
    Err,
}

/// Indicates how a generator function that has been called/resumed should return.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum GeneratorResumeKind {
    Normal,
    Throw,
    Return,
}
