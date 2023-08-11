//! `CallFrame`
//!
//! This module will provides everything needed to implement the `CallFrame`

use crate::{
    builtins::{iterable::IteratorRecord, promise::PromiseCapability},
    environments::BindingLocator,
    object::JsObject,
    vm::CodeBlock,
    JsValue,
};
use boa_gc::{Finalize, Gc, Trace};
use thin_vec::ThinVec;

/// A `CallFrame` holds the state of a function call.
#[derive(Clone, Debug, Finalize, Trace)]
pub struct CallFrame {
    pub(crate) code_block: Gc<CodeBlock>,
    pub(crate) pc: u32,
    pub(crate) fp: u32,
    pub(crate) env_fp: u32,
    // Tracks the number of environments in environment entry.
    // On abrupt returns this is used to decide how many environments need to be pop'ed.
    pub(crate) argument_count: u32,
    pub(crate) promise_capability: Option<PromiseCapability>,

    // When an async generator is resumed, the generator object is needed
    // to fulfill the steps 4.e-j in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart).
    pub(crate) async_generator: Option<JsObject>,

    // Iterators and their `[[Done]]` flags that must be closed when an abrupt completion is thrown.
    pub(crate) iterators: ThinVec<IteratorRecord>,

    // The stack of bindings being updated.
    pub(crate) binding_stack: Vec<BindingLocator>,

    /// How many iterations a loop has done.
    pub(crate) loop_iteration_count: u64,
}

/// ---- `CallFrame` public API ----
impl CallFrame {
    /// Retrieves the [`CodeBlock`] of this call frame.
    #[inline]
    #[must_use]
    pub const fn code_block(&self) -> &Gc<CodeBlock> {
        &self.code_block
    }
}

/// ---- `CallFrame` creation methods ----
impl CallFrame {
    /// Creates a new `CallFrame` with the provided `CodeBlock`.
    pub(crate) fn new(code_block: Gc<CodeBlock>) -> Self {
        Self {
            code_block,
            pc: 0,
            fp: 0,
            env_fp: 0,
            argument_count: 0,
            promise_capability: None,
            async_generator: None,
            iterators: ThinVec::new(),
            binding_stack: Vec::new(),
            loop_iteration_count: 0,
        }
    }

    /// Updates a `CallFrame`'s `argument_count` field with the value provided.
    pub(crate) fn with_argument_count(mut self, count: u32) -> Self {
        self.argument_count = count;
        self
    }

    /// Updates a `CallFrame`'s `env_fp` field with the value provided.
    pub(crate) fn with_env_fp(mut self, env_fp: u32) -> Self {
        self.env_fp = env_fp;
        self
    }
}

/// ---- `CallFrame` stack methods ----
impl CallFrame {
    pub(crate) fn set_frame_pointer(&mut self, pointer: u32) {
        self.fp = pointer;
    }
}

/// Indicates how a generator function that has been called/resumed should return.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(u8)]
pub(crate) enum GeneratorResumeKind {
    #[default]
    Normal = 0,
    Throw,
    Return,
}

impl From<GeneratorResumeKind> for JsValue {
    fn from(value: GeneratorResumeKind) -> Self {
        Self::new(value as u8)
    }
}

impl JsValue {
    /// Convert value to [`GeneratorResumeKind`].
    ///
    /// # Panics
    ///
    /// If not a integer type or not in the range `1..=2`.
    #[track_caller]
    pub(crate) fn to_generator_resume_kind(&self) -> GeneratorResumeKind {
        if let Self::Integer(value) = self {
            match *value {
                0 => return GeneratorResumeKind::Normal,
                1 => return GeneratorResumeKind::Throw,
                2 => return GeneratorResumeKind::Return,
                _ => unreachable!("generator kind must be a integer between 1..=2, got {value}"),
            }
        }

        unreachable!("generator kind must be a integer type")
    }
}
