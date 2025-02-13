use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
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
    fn operation(
        exit: u32,
        lhs: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let lhs = registers.get(lhs);
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

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.read::<u8>().into();
        Self::operation(exit, lhs, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.read::<u16>().into();
        Self::operation(exit, lhs, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.read::<u32>();
        Self::operation(exit, lhs, registers, context)
    }
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
    fn operation(
        exit: u32,
        lhs: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let lhs = registers.get(lhs);
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

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let exit = context.vm.read::<u32>();
        let lhs = u32::from(context.vm.read::<u8>());
        Self::operation(exit, lhs, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let exit = context.vm.read::<u32>();
        let lhs = u32::from(context.vm.read::<u16>());
        Self::operation(exit, lhs, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.read::<u32>();
        Self::operation(exit, lhs, registers, context)
    }
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
    fn operation(
        exit: u32,
        lhs: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let lhs = registers.get(lhs);
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

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let exit = context.vm.read::<u32>();
        let lhs = u32::from(context.vm.read::<u8>());
        Self::operation(exit, lhs, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let exit = context.vm.read::<u32>();
        let lhs = u32::from(context.vm.read::<u16>());
        Self::operation(exit, lhs, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.read::<u32>();
        Self::operation(exit, lhs, registers, context)
    }
}
