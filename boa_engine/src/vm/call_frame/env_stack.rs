//! Module for implementing a `CallFrame`'s environment stacks

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum EnvEntryKind {
    Global,
    Loop,
    Try,
    Catch,
    Finally,
}
/// Tracks the number of environments in the current try-catch-finally block.
///
/// Because of the interactions between loops and try-catch-finally blocks,
/// the number of loop blocks in the try-catch-finally block also needs to be tracked.
#[derive(Copy, Clone, Debug)]
pub(crate) struct EnvStackEntry {
    kind: EnvEntryKind,
    env_num: usize,
}

impl Default for EnvStackEntry {
    fn default() -> Self {
        Self {
            kind: EnvEntryKind::Global,
            env_num: 0,
        }
    }
}

/// ---- `EnvStackEntry` creation methods ----
impl EnvStackEntry {
    pub(crate) const fn with_try_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Try;
        self
    }

    pub(crate) const fn with_loop_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Loop;
        self
    }

    pub(crate) const fn with_catch_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Catch;
        self
    }

    pub(crate) const fn with_finally_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Finally;
        self
    }

    pub(crate) const fn with_initial_env_num(mut self, value: usize) -> Self {
        self.env_num = value;
        self
    }
}

/// ---- `EnvStackEntry` interaction methods ----
impl EnvStackEntry {
    /// Returns true if an `EnvStackEntry` is a loop
    pub(crate) fn is_loop_env(&self) -> bool {
        self.kind == EnvEntryKind::Loop
    }

    /// Returns true if an `EnvStackEntry` is a try block
    pub(crate) fn is_try_env(&self) -> bool {
        self.kind == EnvEntryKind::Try
    }

    /// Returns the current environment number for this entry.
    pub(crate) const fn env_num(&self) -> usize {
        self.env_num
    }

    /// Checks if a env block should be popped
    pub(crate) fn should_be_popped(&self) -> bool {
        self.kind != EnvEntryKind::Global && self.env_num == 0
    }

    pub(crate) fn inc_env_num(&mut self) {
        self.env_num += 1;
    }

    pub(crate) fn dec_env_num(&mut self) {
        self.env_num -= 1;
    }
}