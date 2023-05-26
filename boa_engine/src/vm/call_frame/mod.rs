//! `CallFrame`
//!
//! This module will provides everything needed to implement the `CallFrame`

mod abrupt_record;
mod env_stack;

use crate::{
    builtins::promise::PromiseCapability, environments::BindingLocator, object::JsObject,
    vm::CodeBlock,
};
use boa_gc::{Finalize, Gc, Trace};
use thin_vec::ThinVec;

pub(crate) use abrupt_record::AbruptCompletionRecord;
pub(crate) use env_stack::EnvStackEntry;

/// A `CallFrame` holds the state of a function call.
#[derive(Clone, Debug, Finalize, Trace)]
pub struct CallFrame {
    pub(crate) code_block: Gc<CodeBlock>,
    pub(crate) pc: u32,
    pub(crate) fp: u32,
    #[unsafe_ignore_trace]
    pub(crate) abrupt_completion: Option<AbruptCompletionRecord>,
    #[unsafe_ignore_trace]
    pub(crate) r#yield: bool,
    pub(crate) pop_on_return: u32,
    // Tracks the number of environments in environment entry.
    // On abrupt returns this is used to decide how many environments need to be pop'ed.
    pub(crate) env_stack: Vec<EnvStackEntry>,
    pub(crate) argument_count: u32,
    #[unsafe_ignore_trace]
    pub(crate) generator_resume_kind: GeneratorResumeKind,
    pub(crate) promise_capability: Option<PromiseCapability>,

    // When an async generator is resumed, the generator object is needed
    // to fulfill the steps 4.e-j in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart).
    pub(crate) async_generator: Option<JsObject>,

    // Iterators and their `[[Done]]` flags that must be closed when an abrupt completion is thrown.
    pub(crate) iterators: ThinVec<(JsObject, bool)>,

    // The stack of bindings being updated.
    pub(crate) binding_stack: Vec<BindingLocator>,
}

/// ---- `CallFrame` public API ----
impl CallFrame {
    /// Retrieves the [`CodeBlock`] of this call frame.
    #[inline]
    pub const fn code_block(&self) -> &Gc<CodeBlock> {
        &self.code_block
    }
}

/// ---- `CallFrame` creation methods ----
impl CallFrame {
    /// Creates a new `CallFrame` with the provided `CodeBlock`.
    pub(crate) fn new(code_block: Gc<CodeBlock>) -> Self {
        let max_length = code_block.bytecode.len() as u32;
        Self {
            code_block,
            pc: 0,
            fp: 0,
            pop_on_return: 0,
            env_stack: Vec::from([EnvStackEntry::new(0, max_length)]),
            abrupt_completion: None,
            r#yield: false,
            argument_count: 0,
            generator_resume_kind: GeneratorResumeKind::Normal,
            promise_capability: None,
            async_generator: None,
            iterators: ThinVec::new(),
            binding_stack: Vec::new(),
        }
    }

    /// Updates a `CallFrame`'s `argument_count` field with the value provided.
    pub(crate) fn with_argument_count(mut self, count: u32) -> Self {
        self.argument_count = count;
        self
    }
}

/// ---- `CallFrame` stack methods ----
impl CallFrame {
    pub(crate) fn set_frame_pointer(&mut self, pointer: u32) {
        self.fp = pointer;
    }

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

/// Indicates how a generator function that has been called/resumed should return.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum GeneratorResumeKind {
    Normal,
    Throw,
    Return,
}
