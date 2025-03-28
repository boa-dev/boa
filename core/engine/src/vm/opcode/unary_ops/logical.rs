use crate::{
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionType, Registers,
    },
    Context, JsResult,
};

/// `LogicalNot` implements the Opcode Operation for `Opcode::LogicalNot`
///
/// Operation:
///  - Unary logical `!` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogicalNot;

impl LogicalNot {
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn operation(
        value: VaryingOperand,
        registers: &mut Registers,
        _: &mut Context,
    ) -> JsResult<CompletionType> {
        registers.set(
            value.into(),
            (!registers.get(value.into()).to_boolean()).into(),
        );
        Ok(CompletionType::Normal)
    }
}

impl Operation for LogicalNot {
    const NAME: &'static str = "LogicalNot";
    const INSTRUCTION: &'static str = "INST - LogicalNot";
    const COST: u8 = 1;
}
