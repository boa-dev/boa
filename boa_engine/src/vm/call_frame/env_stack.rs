//! Module for implementing a `CallFrame`'s environment stacks

#[derive(Clone, Debug)]
pub(crate) enum EnvEntryKind {
    Global,
    Loop {
        /// How many iterations a loop has done.
        iteration_count: u64,

        /// The index of the currently active iterator.
        iterator: Option<u32>,
    },
    Try {
        /// The length of the value stack when the try block was entered.
        ///
        /// This is used to pop exact amount values from the stack
        /// when a throw happens.
        fp: u32,
    },
    Finally,
    Labelled,
}

/// The `EnvStackEntry` tracks the environment count and relevant information for the current environment.
#[derive(Clone, Debug)]
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
    /// Creates a new [`EnvStackEntry`] with the supplied start addresses.
    pub(crate) const fn new(start_address: u32, exit_address: u32) -> Self {
        Self {
            start: start_address,
            exit: exit_address,
            kind: EnvEntryKind::Global,
            env_num: 0,
        }
    }

    /// Returns calling [`EnvStackEntry`] with `kind` field of [`EnvEntryKind::Try`].
    pub(crate) const fn with_try_flag(mut self, fp: u32) -> Self {
        self.kind = EnvEntryKind::Try { fp };
        self
    }

    /// Returns calling [`EnvStackEntry`] with `kind` field of [`EnvEntryKind::Loop`], loop iteration set to zero
    /// and iterator index set to `iterator`.
    pub(crate) const fn with_iterator_loop_flag(
        mut self,
        iteration_count: u64,
        iterator: u32,
    ) -> Self {
        self.kind = EnvEntryKind::Loop {
            iteration_count,
            iterator: Some(iterator),
        };
        self
    }

    /// Returns calling [`EnvStackEntry`] with `kind` field of [`EnvEntryKind::Loop`].
    /// And the loop iteration set to zero.
    pub(crate) const fn with_loop_flag(mut self, iteration_count: u64) -> Self {
        self.kind = EnvEntryKind::Loop {
            iteration_count,
            iterator: None,
        };
        self
    }

    /// Returns calling [`EnvStackEntry`] with `kind` field of [`EnvEntryKind::Finally`].
    pub(crate) const fn with_finally_flag(mut self) -> Self {
        self.kind = EnvEntryKind::Finally;
        self
    }

    /// Returns calling [`EnvStackEntry`] with `kind` field of [`EnvEntryKind::Labelled`].
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

    /// Returns the `fp` field of this [`EnvEntryKind::Try`].
    ///
    /// # Panics
    ///
    /// If this [`EnvStackEntry`] is **not** a [`EnvEntryKind::Try`].
    pub(crate) fn try_env_frame_pointer(&self) -> u32 {
        if let EnvEntryKind::Try { fp } = &self.kind {
            return *fp;
        }
        unreachable!("trying to get frame pointer of a non-try environment")
    }

    pub(crate) const fn is_global_env(&self) -> bool {
        matches!(self.kind, EnvEntryKind::Global)
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

    /// Returns the active iterator index if `EnvStackEntry` is an iterator loop.
    pub(crate) const fn iterator(&self) -> Option<u32> {
        if let EnvEntryKind::Loop { iterator, .. } = self.kind {
            return iterator;
        }
        None
    }

    /// Returns true if an `EnvStackEntry` is a try block
    pub(crate) const fn is_try_env(&self) -> bool {
        matches!(self.kind, EnvEntryKind::Try { .. })
    }

    /// Returns true if an `EnvStackEntry` is a labelled block
    pub(crate) const fn is_labelled_env(&self) -> bool {
        matches!(self.kind, EnvEntryKind::Labelled)
    }

    pub(crate) const fn is_finally_env(&self) -> bool {
        matches!(self.kind, EnvEntryKind::Finally)
    }

    pub(crate) const fn is_loop_env(&self) -> bool {
        matches!(self.kind, EnvEntryKind::Loop { .. })
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
}
