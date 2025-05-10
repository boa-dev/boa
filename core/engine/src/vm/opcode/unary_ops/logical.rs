use crate::{
    vm::{
        opcode::{Operation, VaryingOperand},
        Registers,
    },
    Context,
};

/// `LogicalNot` implements the Opcode Operation for `Opcode::LogicalNot`
///
/// Operation:
///  - Unary logical `!` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogicalNot;

impl LogicalNot {
    #[inline(always)]
    pub(crate) fn operation(value: VaryingOperand, registers: &mut Registers, _: &mut Context) {
        registers.set(
            value.into(),
            (!registers.get(value.into()).to_boolean()).into(),
        );
    }
}

impl Operation for LogicalNot {
    const NAME: &'static str = "LogicalNot";
    const INSTRUCTION: &'static str = "INST - LogicalNot";
    const COST: u8 = 1;
}
