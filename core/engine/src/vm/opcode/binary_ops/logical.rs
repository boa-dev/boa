use crate::{
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionType, Registers,
    },
    Context, JsResult,
};

/// `LogicalAnd` implements the Opcode Operation for `Opcode::LogicalAnd`
///
/// Operation:
///  - Binary logical `&&` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogicalAnd;

impl LogicalAnd {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    #[inline(always)]
    pub(crate) fn operation(
        (exit, lhs): (u32, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let lhs = registers.get(lhs.into());
        if !lhs.to_boolean() {
            context.vm.frame_mut().pc = exit;
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for LogicalAnd {
    const NAME: &'static str = "LogicalAnd";
    const INSTRUCTION: &'static str = "INST - LogicalAnd";
    const COST: u8 = 1;
}

/// `LogicalOr` implements the Opcode Operation for `Opcode::LogicalOr`
///
/// Operation:
///  - Binary logical `||` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogicalOr;

impl LogicalOr {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    #[inline(always)]
    pub(crate) fn operation(
        (exit, lhs): (u32, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let lhs = registers.get(lhs.into());
        if lhs.to_boolean() {
            context.vm.frame_mut().pc = exit;
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for LogicalOr {
    const NAME: &'static str = "LogicalOr";
    const INSTRUCTION: &'static str = "INST - LogicalOr";
    const COST: u8 = 1;
}

/// `Coalesce` implements the Opcode Operation for `Opcode::Coalesce`
///
/// Operation:
///  - Binary logical `||` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct Coalesce;

impl Coalesce {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    #[inline(always)]
    pub(crate) fn operation(
        (exit, lhs): (u32, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let lhs = registers.get(lhs.into());
        if !lhs.is_null_or_undefined() {
            context.vm.frame_mut().pc = exit;
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for Coalesce {
    const NAME: &'static str = "Coalesce";
    const INSTRUCTION: &'static str = "INST - Coalesce";
    const COST: u8 = 1;
}
