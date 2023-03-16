//! Implements an `AbruptCompletionRecord` struct for `CallFrame` tracking.

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum AbruptKind {
    Continue,
    Break,
    Throw,
    Return,
}

/// The `AbruptCompletionRecord` tracks the current `AbruptCompletion` and target address of completion.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AbruptCompletionRecord {
    kind: AbruptKind,
    target: u32,
}

/// ---- `AbruptCompletionRecord` initialization methods ----
impl AbruptCompletionRecord {
    /// Creates an `AbruptCompletionRecord` for an abrupt `Break`.
    pub(crate) const fn new_break() -> Self {
        Self {
            kind: AbruptKind::Break,
            target: u32::MAX,
        }
    }

    /// Creates an `AbruptCompletionRecord` for an abrupt `Continue`.
    pub(crate) const fn new_continue() -> Self {
        Self {
            kind: AbruptKind::Continue,
            target: u32::MAX,
        }
    }

    /// Creates an `AbruptCompletionRecord` for an abrupt `Throw`.
    pub(crate) const fn new_throw() -> Self {
        Self {
            kind: AbruptKind::Throw,
            target: u32::MAX,
        }
    }

    /// Creates an `AbruptCompletionRecord` for an abrupt `Return`.
    pub(crate) const fn new_return() -> Self {
        Self {
            kind: AbruptKind::Return,
            target: u32::MAX,
        }
    }

    /// Set the target field of the `AbruptCompletionRecord`.
    pub(crate) const fn with_initial_target(mut self, target: u32) -> Self {
        self.target = target;
        self
    }
}

/// ---- `AbruptCompletionRecord` interaction methods ----
impl AbruptCompletionRecord {
    /// Returns a boolean value for whether `AbruptCompletionRecord` is a break.
    pub(crate) fn is_break(self) -> bool {
        self.kind == AbruptKind::Break
    }

    /// Returns a boolean value for whether `AbruptCompletionRecord` is a continue.
    pub(crate) fn is_continue(self) -> bool {
        self.kind == AbruptKind::Continue
    }

    /// Returns a boolean value for whether `AbruptCompletionRecord` is a return.
    pub(crate) fn is_return(self) -> bool {
        self.kind == AbruptKind::Return
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
