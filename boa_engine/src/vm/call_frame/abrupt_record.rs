//! Implements an `AbruptCompletionRecord` struct for `CallFrame` tracking.

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum AbruptKind {
    None,
    Continue,
    Break,
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
            kind: AbruptKind::None,
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

    /// Set the target field of the `AbruptCompletionRecord`.
    pub(crate) const fn with_initial_target(mut self, target: u32) -> Self {
        self.target = target;
        self
    }
}

impl AbruptCompletionRecord {
    /// Returns bool if `AbruptCompletionRecord` is a break.
    pub(crate) fn is_break(self) -> bool {
        self.kind == AbruptKind::Break
    }

    /// Returns bool if `AbruptCompletionRecord` is a continue.
    pub(crate) fn is_continue(self) -> bool {
        self.kind == AbruptKind::Continue
    }

    /// Returns the value of `AbruptCompletionRecord`'s `target` field.
    pub(crate) const fn target(self) -> u32 {
        self.target
    }
}
