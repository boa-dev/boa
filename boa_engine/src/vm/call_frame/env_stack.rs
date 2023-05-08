//! Module for implementing a `CallFrame`'s environment stacks

use crate::JsValue;
use boa_gc::{Finalize, Trace};

#[derive(Clone, Debug, Finalize, Trace)]
pub(crate) enum EnvEntryKind {
    Global,
    Loop {
        /// This is used to keep track of how many iterations a loop has done.
        iteration_count: u64,

        // This is the latest return value of the loop.
        value: JsValue,
    },
    Try,
    Catch,
    Finally,
    Labelled,
}

impl PartialEq for EnvEntryKind {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Global, Self::Global)
                | (Self::Loop { .. }, Self::Loop { .. })
                | (Self::Try, Self::Try)
                | (Self::Catch, Self::Catch)
                | (Self::Finally, Self::Finally)
                | (Self::Labelled, Self::Labelled)
        )
    }
}

/// The `EnvStackEntry` tracks the environment count and relevant information for the current environment.
#[derive(Clone, Debug, Finalize, Trace)]
pub(crate) struct EnvStackEntry {
    start: u32,
    exit: u32,
    kind: EnvEntryKind,
    env_num: usize,
}

impl Default for EnvStackEntry {
    fn default() -> Self {
        Self {
            start: 0,
            exit: u32::MAX,
            kind: EnvEntryKind::Global,
            env_num: 0,
        }
    }
}

/// ---- `EnvStackEntry` creation methods ----
impl EnvStackEntry {
    /// Creates a new `EnvStackEntry` with the supplied start addresses.
    pub(crate) const fn new(start_address: u32, exit_address: u32) -> Self {
        Self {
            start: start_address,
            exit: exit_address,
            kind: EnvEntryKind::Global,
            env_num: 0,
        }
    }

    /// Returns calling `EnvStackEntry` with `kind` field of `Try`.
    pub(crate) fn with_try_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Try;
        self
    }

    /// Returns calling `EnvStackEntry` with `kind` field of `Loop`.
    /// And the loop iteration set to zero.
    pub(crate) fn with_loop_flag(mut self, iteration_count: u64) -> Self {
        self.kind = EnvEntryKind::Loop {
            iteration_count,
            value: JsValue::undefined(),
        };
        self
    }

    /// Returns calling `EnvStackEntry` with `kind` field of `Catch`.
    pub(crate) fn with_catch_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Catch;
        self
    }

    /// Returns calling `EnvStackEntry` with `kind` field of `Finally`.
    pub(crate) fn with_finally_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Finally;
        self
    }

    /// Returns calling `EnvStackEntry` with `kind` field of `Labelled`.
    pub(crate) fn with_labelled_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Labelled;
        self
    }

    pub(crate) const fn with_start_address(mut self, start_address: u32) -> Self {
        self.start = start_address;
        self
    }
}

/// ---- `EnvStackEntry` interaction methods ----
impl EnvStackEntry {
    /// Returns the `start` field of this `EnvStackEntry`.
    pub(crate) const fn start_address(&self) -> u32 {
        self.start
    }

    /// Returns the `exit` field of this `EnvStackEntry`.
    pub(crate) const fn exit_address(&self) -> u32 {
        self.exit
    }

    pub(crate) fn is_global_env(&self) -> bool {
        self.kind == EnvEntryKind::Global
    }

    /// Returns the loop iteration count if `EnvStackEntry` is a loop.
    pub(crate) const fn as_loop_iteration_count(&self) -> Option<u64> {
        if let EnvEntryKind::Loop {
            iteration_count, ..
        } = self.kind
        {
            return Some(iteration_count);
        }
        None
    }

    /// Increases loop iteration count if `EnvStackEntry` is a loop.
    pub(crate) fn increase_loop_iteration_count(&mut self) {
        if let EnvEntryKind::Loop {
            iteration_count, ..
        } = &mut self.kind
        {
            *iteration_count = iteration_count.wrapping_add(1);
        }
    }

    /// Returns the loop return value if `EnvStackEntry` is a loop.
    pub(crate) const fn loop_env_value(&self) -> Option<&JsValue> {
        if let EnvEntryKind::Loop { value, .. } = &self.kind {
            return Some(value);
        }
        None
    }

    /// Returns true if an `EnvStackEntry` is a try block
    pub(crate) fn is_try_env(&self) -> bool {
        self.kind == EnvEntryKind::Try
    }

    /// Returns true if an `EnvStackEntry` is a labelled block
    pub(crate) fn is_labelled_env(&self) -> bool {
        self.kind == EnvEntryKind::Labelled
    }

    /// Returns true if an `EnvStackEntry` is a catch block
    pub(crate) fn is_catch_env(&self) -> bool {
        self.kind == EnvEntryKind::Catch
    }

    pub(crate) fn is_finally_env(&self) -> bool {
        self.kind == EnvEntryKind::Finally
    }

    /// Returns the current environment number for this entry.
    pub(crate) const fn env_num(&self) -> usize {
        self.env_num
    }

    pub(crate) fn set_exit_address(&mut self, exit_address: u32) {
        self.exit = exit_address;
    }

    pub(crate) fn clear_env_num(&mut self) {
        self.env_num = 0;
    }

    /// Increments the `env_num` field for current `EnvEntryStack`.
    pub(crate) fn inc_env_num(&mut self) {
        (self.env_num, _) = self.env_num.overflowing_add(1);
    }

    /// Decrements the `env_num` field for current `EnvEntryStack`.
    pub(crate) fn dec_env_num(&mut self) {
        (self.env_num, _) = self.env_num.overflowing_sub(1);
    }

    /// Set the loop return value for the current `EnvStackEntry`.
    pub(crate) fn set_loop_return_value(&mut self, value: JsValue) -> bool {
        if let EnvEntryKind::Loop { value: v, .. } = &mut self.kind {
            *v = value;
            true
        } else {
            false
        }
    }
}
