//! `CallFrame`
//!
//! This module will provides everything needed to implement the `CallFrame`

use crate::{
    JsValue, builtins::iterable::IteratorRecord, environments::EnvironmentStack, realm::Realm,
    vm::CodeBlock,
};
use boa_ast::scope::BindingLocator;
use boa_gc::{Finalize, Gc, Trace};
use thin_vec::ThinVec;

use super::ActiveRunnable;

bitflags::bitflags! {
    /// Flags associated with a [`CallFrame`].
    #[derive(Debug, Default, Clone, Copy)]
    pub(crate) struct CallFrameFlags: u8 {
        /// When we return from this [`CallFrame`] to stop execution and
        /// return from [`crate::Context::run()`], and leave the remaining [`CallFrame`]s unchanged.
        const EXIT_EARLY = 0b0000_0001;

        /// Was this [`CallFrame`] created from the `__construct__()` internal object method?
        const CONSTRUCT = 0b0000_0010;

        /// Does this [`CallFrame`] need to push registers on [`Vm::push_frame()`].
        const REGISTERS_ALREADY_PUSHED = 0b0000_0100;

        /// If the `this` value has been cached.
        const THIS_VALUE_CACHED = 0b0000_1000;
    }
}

/// A `CallFrame` holds the state of a function call.
#[derive(Clone, Debug, Finalize, Trace)]
pub struct CallFrame {
    pub(crate) code_block: Gc<CodeBlock>,
    pub(crate) pc: u32,
    /// The register pointer, points to the first register in the stack.
    ///
    // TODO: Check if storing the frame pointer instead of argument count and computing the
    //       argument count based on the pointers would be better for accessing the arguments
    //       and the elements before the register pointer.
    pub(crate) rp: u32,
    pub(crate) argument_count: u32,
    pub(crate) env_fp: u32,

    // Iterators and their `[[Done]]` flags that must be closed when an abrupt completion is thrown.
    pub(crate) iterators: ThinVec<IteratorRecord>,

    // The stack of bindings being updated.
    // SAFETY: Nothing in `BindingLocator` requires tracing, so this is safe.
    #[unsafe_ignore_trace]
    pub(crate) binding_stack: Vec<BindingLocator>,

    /// How many iterations a loop has done.
    pub(crate) loop_iteration_count: u64,

    /// `[[ScriptOrModule]]`
    pub(crate) active_runnable: Option<ActiveRunnable>,

    /// \[\[Environment\]\]
    pub(crate) environments: EnvironmentStack,

    /// \[\[Realm\]\]
    pub(crate) realm: Realm,

    // SAFETY: Nothing in `CallFrameFlags` requires tracing, so this is safe.
    #[unsafe_ignore_trace]
    pub(crate) flags: CallFrameFlags,
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
    pub(crate) const FUNCTION_PROLOGUE: u32 = 2;
    const THIS_POSITION: usize = 2;
    const FUNCTION_POSITION: usize = 1;
    pub(crate) const PROMISE_CAPABILITY_PROMISE_REGISTER_INDEX: usize = 0;
    pub(crate) const PROMISE_CAPABILITY_RESOLVE_REGISTER_INDEX: usize = 1;
    pub(crate) const PROMISE_CAPABILITY_REJECT_REGISTER_INDEX: usize = 2;
    pub(crate) const ASYNC_GENERATOR_OBJECT_REGISTER_INDEX: usize = 3;

    /// Creates a new `CallFrame` with the provided `CodeBlock`.
    pub(crate) fn new(
        code_block: Gc<CodeBlock>,
        active_runnable: Option<ActiveRunnable>,
        environments: EnvironmentStack,
        realm: Realm,
    ) -> Self {
        Self {
            pc: 0,
            rp: 0,
            env_fp: 0,
            argument_count: 0,
            iterators: ThinVec::new(),
            binding_stack: Vec::new(),
            code_block,
            loop_iteration_count: 0,
            active_runnable,
            environments,
            realm,
            flags: CallFrameFlags::empty(),
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

    /// Updates a `CallFrame`'s `flags` field with the value provided.
    pub(crate) fn with_flags(mut self, flags: CallFrameFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Returns the index of `this` in the stack.
    pub(crate) fn this_index(&self) -> usize {
        self.rp as usize - self.argument_count as usize - Self::THIS_POSITION
    }

    /// Returns the index of the function in the stack.
    pub(crate) fn function_index(&self) -> usize {
        self.rp as usize - self.argument_count as usize - Self::FUNCTION_POSITION
    }

    /// Returns the index of the promise capability promise register in the stack.
    pub(crate) fn promise_capability_promise_register_index(&self) -> usize {
        self.rp as usize + Self::PROMISE_CAPABILITY_PROMISE_REGISTER_INDEX
    }

    /// Returns the index of the promise capability resolve register in the stack.
    pub(crate) fn promise_capability_resolve_register_index(&self) -> usize {
        self.rp as usize + Self::PROMISE_CAPABILITY_RESOLVE_REGISTER_INDEX
    }

    /// Returns the index of the promise capability reject register in the stack.
    pub(crate) fn promise_capability_reject_register_index(&self) -> usize {
        self.rp as usize + Self::PROMISE_CAPABILITY_REJECT_REGISTER_INDEX
    }

    /// Returns the index of the async generator object register in the stack.
    pub(crate) fn async_generator_object_register_index(&self) -> usize {
        self.rp as usize + Self::ASYNC_GENERATOR_OBJECT_REGISTER_INDEX
    }

    /// Returns the range of the arguments in the stack.
    pub(crate) fn arguments_range(&self) -> std::ops::Range<usize> {
        (self.rp as usize - self.argument_count as usize)..self.rp as usize
    }

    /// Returns the frame pointer of this `CallFrame`.
    pub(crate) fn frame_pointer(&self) -> usize {
        (self.rp - self.argument_count - Self::FUNCTION_PROLOGUE) as usize
    }

    /// Does this have the [`CallFrameFlags::EXIT_EARLY`] flag.
    pub(crate) fn exit_early(&self) -> bool {
        self.flags.contains(CallFrameFlags::EXIT_EARLY)
    }

    /// Set the [`CallFrameFlags::EXIT_EARLY`] flag.
    pub(crate) fn set_exit_early(&mut self, early_exit: bool) {
        self.flags.set(CallFrameFlags::EXIT_EARLY, early_exit);
    }

    /// Does this have the [`CallFrameFlags::CONSTRUCT`] flag.
    pub(crate) fn construct(&self) -> bool {
        self.flags.contains(CallFrameFlags::CONSTRUCT)
    }

    /// Does this [`CallFrame`] need to push registers on [`super::Vm::push_frame()`].
    pub(crate) fn registers_already_pushed(&self) -> bool {
        self.flags
            .contains(CallFrameFlags::REGISTERS_ALREADY_PUSHED)
    }

    /// Does this [`CallFrame`] have a cached `this` value.
    ///
    /// The cached value is placed in the [`CallFrame::THIS_POSITION`] position.
    pub(crate) fn has_this_value_cached(&self) -> bool {
        self.flags.contains(CallFrameFlags::THIS_VALUE_CACHED)
    }
}

/// ---- `CallFrame` stack methods ----
impl CallFrame {
    pub(crate) fn set_register_pointer(&mut self, pointer: u32) {
        self.rp = pointer;
    }
}

/// Indicates how a generator function that has been called/resumed should return.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum GeneratorResumeKind {
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
        if let Some(value) = self.as_i32() {
            match value {
                0 => return GeneratorResumeKind::Normal,
                1 => return GeneratorResumeKind::Throw,
                2 => return GeneratorResumeKind::Return,
                _ => unreachable!("generator kind must be a integer between 1..=2, got {value}"),
            }
        }

        unreachable!("generator kind must be a integer type")
    }
}
