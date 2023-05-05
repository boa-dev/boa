//! Module for implementing a `CallFrame`'s environment stacks

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum EnvEntryKind {
    Global,
    Loop {
        /// This is used to keep track of how many iterations a loop has done.
        iteration_count: u64,
    },
    Try,
    Catch,
    Finally,
    Labelled,
}

/// The `EnvStackEntry` tracks the environment count and relavant information for the current environment.
#[derive(Copy, Clone, Debug)]
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
    pub(crate) const fn with_try_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Try;
        self
    }

    /// Returns calling `EnvStackEntry` with `kind` field of `Loop`.
    /// And the loop iteration set to zero.
    pub(crate) const fn with_loop_flag(mut self, iteration_count: u64) -> Self {
        self.kind = EnvEntryKind::Loop { iteration_count };
        self
    }

    /// Returns calling `EnvStackEntry` with `kind` field of `Catch`.
    pub(crate) const fn with_catch_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Catch;
        self
    }

    /// Returns calling `EnvStackEntry` with `kind` field of `Finally`.
    pub(crate) const fn with_finally_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Finally;
        self
    }

    /// Returns calling `EnvStackEntry` with `kind` field of `Labelled`.
    pub(crate) const fn with_labelled_flag(mut self) -> Self {
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

    /// Returns true if an `EnvStackEntry` is a loop
    pub(crate) const fn is_loop_env(&self) -> bool {
        matches!(self.kind, EnvEntryKind::Loop { .. })
    }

    pub(crate) const fn as_loop_iteration_count(self) -> Option<u64> {
        if let EnvEntryKind::Loop { iteration_count } = self.kind {
            return Some(iteration_count);
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
        self.env_num += 1;
    }

    /// Decrements the `env_num` field for current `EnvEntryStack`.
    pub(crate) fn dec_env_num(&mut self) {
        self.env_num -= 1;
    }
}
