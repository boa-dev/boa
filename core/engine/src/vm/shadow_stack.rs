use std::fmt::{Display, Write};

use boa_gc::{Finalize, Trace};
use boa_string::JsString;
use thin_vec::ThinVec;

use super::source_info::{NativeSourceInfo, SourceInfo};

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
    Native {
        function_name: Option<JsString>,
        source_info: NativeSourceInfo,
    },
    Bytecode {
        pc: u32,
        source_info: SourceInfo,
    },
}

impl Display for ShadowEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShadowEntry::Native {
                function_name,
                source_info,
            } => {
                if function_name.is_some() || source_info.as_location().is_some() {
                    f.write_str(" (native")?;
                    if let Some(function_name) = function_name {
                        write!(f, " {}", function_name.to_std_string_escaped())?;
                    }
                    if let Some(location) = source_info.as_location() {
                        write!(f, " at {location}")?;
                    }
                    f.write_char(')')?;
                }
            }
            ShadowEntry::Bytecode { pc, source_info } => {
                let path = source_info.map().path();
                let position = source_info.map().find(*pc);

                if path.is_some() || position.is_some() {
                    write!(f, " ({}", source_info.map().path())?;

                    if let Some(position) = position {
                        write!(
                            f,
                            ":{}:{}",
                            position.line_number(),
                            position.column_number()
                        )?;
                    }

                    f.write_char(')')?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ShadowStack {
    stack: ThinVec<ShadowEntry>,
}

impl ShadowStack {
    pub(crate) fn push_native(
        &mut self,
        last_pc: u32,
        function_name: JsString,
        native_source_info: NativeSourceInfo,
    ) {
        // NOTE: pc points to the next opcode, so we offset by -1 to put it within range.
        let last_pc = last_pc.saturating_sub(1);

        match self.stack.last_mut() {
            Some(ShadowEntry::Bytecode { pc, .. }) => *pc = last_pc,
            Some(ShadowEntry::Native { source_info, .. }) => *source_info = native_source_info,
            _ => {}
        }
        self.stack.push(ShadowEntry::Native {
            function_name: Some(function_name),
            source_info: native_source_info,
        });
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

    #[cfg(feature = "native-backtrace")]
    pub(crate) fn patch_last_native(&mut self, new_source_info: NativeSourceInfo) {
        let Some(ShadowEntry::Native { source_info, .. }) = self.stack.last_mut() else {
            return;
        };
        *source_info = new_source_info;
    }
}
