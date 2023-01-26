//! Implements an abrupt `CompletionRecord`

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum AbruptKind {
    None,
    Continue,
    Break,
}

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
    pub(crate) const fn with_break_flag(mut self) -> Self {
        self.kind = AbruptKind::Break;
        self
    }

    pub(crate) const fn with_continue_flag(mut self) -> Self {
        self.kind = AbruptKind::Continue;
        self
    }

    pub(crate) const fn with_initial_target(mut self, target: u32) -> Self {
        self.target = target;
        self
    }
}

impl AbruptCompletionRecord {
    pub(crate) fn is_break(&self) -> bool {
        self.kind == AbruptKind::Break
    }

    pub(crate) fn is_continue(&self) -> bool {
        self.kind == AbruptKind::Continue
    }

    pub(crate) const fn target(&self) -> u32 {
        self.target
    }
}
