use crate::{Context, vm::opcode::Operation};

/// `Debugger` implements the Opcode Operation for `Opcode::Debugger`
///
/// Operation:
///  - No-op for now. Emitted for `debugger;` statements so they are
///    represented in bytecode and visible during tracing.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Debugger;

impl Debugger {
    #[inline(always)]
    pub(super) fn operation((): (), _: &mut Context) {}
}

impl Operation for Debugger {
    const NAME: &'static str = "Debugger";
    const INSTRUCTION: &'static str = "INST - Debugger";
    const COST: u8 = 1;
}
