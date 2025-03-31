use crate::{
    vm::{
        opcode::{Operation, VaryingOperand},
        Registers,
    },
    Context,
};

/// `LogicalAnd` implements the Opcode Operation for `Opcode::LogicalAnd`
///
/// Operation:
///  - Binary logical `&&` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogicalAnd;

impl LogicalAnd {
    #[inline(always)]
    pub(crate) fn operation(
        (exit, lhs): (u32, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let lhs = registers.get(lhs.into());
        if !lhs.to_boolean() {
            context.vm.frame_mut().pc = exit;
        }
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
    #[inline(always)]
    pub(crate) fn operation(
        (exit, lhs): (u32, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let lhs = registers.get(lhs.into());
        if lhs.to_boolean() {
            context.vm.frame_mut().pc = exit;
        }
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
    #[inline(always)]
    pub(crate) fn operation(
        (exit, lhs): (u32, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let lhs = registers.get(lhs.into());
        if !lhs.is_null_or_undefined() {
            context.vm.frame_mut().pc = exit;
        }
    }
}

impl Operation for Coalesce {
    const NAME: &'static str = "Coalesce";
    const INSTRUCTION: &'static str = "INST - Coalesce";
    const COST: u8 = 1;
}
