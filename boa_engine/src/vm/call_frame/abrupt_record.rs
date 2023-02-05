//! Implements an `AbruptCompletionRecord` struct for `CallFrame` tracking.

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum AbruptKind {
    Continue,
    Break,
    Throw,
}

/// The `AbruptCompletionRecord` tracks the current `AbruptCompletion` and target address of completion.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AbruptCompletionRecord {
    kind: AbruptKind,
    target: u32,
}

impl Default for AbruptCompletionRecord {
    fn default() -> Self {
        Self {
            kind: AbruptKind::Break,
            target: u32::MAX,
        }
    }
}

/// ---- `AbruptCompletionRecord` initialization methods ----
impl AbruptCompletionRecord {
    /// Sets the `kind` field on `AbruptCompletionRecord` to `Break`.
    pub(crate) const fn with_break_flag(mut self) -> Self {
        self.kind = AbruptKind::Break;
        self
    }

    /// Sets the `kind` field on `AbruptCompletionRecord` to `Continue`.
    pub(crate) const fn with_continue_flag(mut self) -> Self {
        self.kind = AbruptKind::Continue;
        self
    }

    pub(crate) const fn with_throw_flag(mut self) -> Self {
        self.kind = AbruptKind::Throw;
        self
    }

    /// Set the target field of the `AbruptCompletionRecord`.
    pub(crate) const fn with_initial_target(mut self, target: u32) -> Self {
        self.target = target;
        self
    }
}

impl AbruptCompletionRecord {
    /// Returns a boolean value for whether `AbruptCompletionRecord` is a break.
    pub(crate) fn is_break(self) -> bool {
        self.kind == AbruptKind::Break
    }

    /// Returns a boolean value for whether `AbruptCompletionRecord` is a continue.
    pub(crate) fn is_continue(self) -> bool {
        self.kind == AbruptKind::Continue
    }

    /// Returns a boolean value for whether `AbruptCompletionRecord` is a throw.
    pub(crate) fn is_throw(self) -> bool {
        self.kind == AbruptKind::Throw
    }

    pub(crate) fn is_throw_with_target(self) -> bool {
        self.is_throw() && self.target < u32::MAX
    }

    /// Returns the value of `AbruptCompletionRecord`'s `target` field.
    pub(crate) const fn target(self) -> u32 {
        self.target
    }
}
