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
    envs: u32,
}

impl Default for AbruptCompletionRecord {
    fn default() -> Self {
        Self {
            kind: AbruptKind::None,
            target: u32::MAX,
            envs: 0,
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

    pub(crate) const fn with_initial_envs(mut self, envs: u32) -> Self {
        self.envs = envs;
        self
    }
}

impl AbruptCompletionRecord {
    pub(crate) fn is_break(&self) -> bool {
        self.kind == AbruptKind::Break
    }

    pub(crate) const fn target(&self) -> u32 {
        self.target
    }

    pub(crate) const fn envs(&self) -> u32 {
        self.envs
    }
}

/// ---- `AbruptCompletionRecord` interaction methods ----
impl AbruptCompletionRecord {
    pub(crate) fn set_break_flag(&mut self) {
        self.kind = AbruptKind::Break;
    }

    pub(crate) fn set_continue_flag(&mut self) {
        self.kind = AbruptKind::Continue;
    }

    pub(crate) fn set_target(&mut self, target: u32) {
        self.target = target;
    }

    pub(crate) fn set_envs(&mut self, envs: u32) {
        self.envs = envs;
    }

    pub(crate) fn inc_envs(&mut self) {
        self.envs += 1;
    }

    pub(crate) fn dec_envs(&mut self) {
        self.envs -= 1;
    }
}