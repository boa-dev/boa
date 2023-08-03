use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `PushLiteral` implements the Opcode Operation for `Opcode::PushLiteral`
///
/// Operation:
///  - Push literal value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushLiteral;

impl PushLiteral {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let value = context.vm.frame().code_block.literals[index].clone();
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushLiteral {
    const NAME: &'static str = "PushLiteral";
    const INSTRUCTION: &'static str = "INST - PushLiteral";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>();
        Self::operation(context, index as usize)
    }

    fn half_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn wide_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        Self::operation(context, index as usize)
    }
}
