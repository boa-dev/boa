use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `LogicalAnd` implements the Opcode Operation for `Opcode::LogicalAnd`
///
/// Operation:
///  - Binary logical `&&` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogicalAnd;

impl Operation for LogicalAnd {
    const NAME: &'static str = "LogicalAnd";
    const INSTRUCTION: &'static str = "INST - LogicalAnd";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.pop();
        if !lhs.to_boolean() {
            context.vm.frame_mut().pc = exit;
            context.vm.push(lhs);
        }
        Ok(CompletionType::Normal)
    }
}

/// `LogicalOr` implements the Opcode Operation for `Opcode::LogicalOr`
///
/// Operation:
///  - Binary logical `||` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogicalOr;

impl Operation for LogicalOr {
    const NAME: &'static str = "LogicalOr";
    const INSTRUCTION: &'static str = "INST - LogicalOr";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.pop();
        if lhs.to_boolean() {
            context.vm.frame_mut().pc = exit;
            context.vm.push(lhs);
        }
        Ok(CompletionType::Normal)
    }
}

/// `Coalesce` implements the Opcode Operation for `Opcode::Coalesce`
///
/// Operation:
///  - Binary logical `||` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct Coalesce;

impl Operation for Coalesce {
    const NAME: &'static str = "Coalesce";
    const INSTRUCTION: &'static str = "INST - Coalesce";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.pop();
        if !lhs.is_null_or_undefined() {
            context.vm.frame_mut().pc = exit;
            context.vm.push(lhs);
        }
        Ok(CompletionType::Normal)
    }
}
