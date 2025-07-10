use boa_gc::{Finalize, Trace};
use boa_string::JsString;
use thin_vec::ThinVec;

use super::source_info::SourceInfo;

#[derive(Debug, Default, Clone, Trace, Finalize)]
pub(crate) struct Backtrace {
    // SAFETY: Nothing in `ShadowEntry` requires trace, so this is safe.
    #[unsafe_ignore_trace]
    stack: ThinVec<ShadowEntry>,
}

impl Backtrace {
    pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = &ShadowEntry> {
        self.stack.iter()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ShadowEntry {
    Native { function_name: JsString },
    Bytecode { pc: u32, source_info: SourceInfo },
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ShadowStack {
    stack: ThinVec<ShadowEntry>,
}

impl ShadowStack {
    pub(crate) fn push_native(&mut self, last_pc: u32, function_name: JsString) {
        // NOTE: pc points to the next opcode, so we offset by -1 to put it within range.
        let last_pc = last_pc.saturating_sub(1);

        if let Some(ShadowEntry::Bytecode { pc, .. }) = self.stack.last_mut() {
            *pc = last_pc;
        }
        self.stack.push(ShadowEntry::Native { function_name });
    }

    pub(crate) fn push_bytecode(&mut self, last_pc: u32, source_info: SourceInfo) {
        // NOTE: pc points to the next opcode, so we offset by -1 to put it within range.
        let last_pc = last_pc.saturating_sub(1);

        if let Some(ShadowEntry::Bytecode { pc, .. }) = self.stack.last_mut() {
            *pc = last_pc;
        }
        self.stack
            .push(ShadowEntry::Bytecode { pc: 0, source_info });
    }

    pub(crate) fn pop(&mut self) -> Option<ShadowEntry> {
        self.stack.pop()
    }

    pub(crate) fn take(&self, n: usize, last_pc: u32) -> Backtrace {
        let mut stack = self
            .stack
            .iter()
            .rev()
            .take(n)
            .rev()
            .cloned()
            .collect::<ThinVec<_>>();

        if let Some(ShadowEntry::Bytecode { pc, .. }) = stack.last_mut() {
            // NOTE: pc points to the next opcode, so we offset by -1 to put it within range.
            *pc = last_pc.saturating_sub(1);
        }
        Backtrace { stack }
    }

    pub(crate) fn caller_position(&self) -> Option<ShadowEntry> {
        // NOTE: We push the function that is currently execution, so the second last is the caller.
        let index = self.stack.len().checked_sub(2)?;
        self.stack.get(index).cloned()
    }
}
