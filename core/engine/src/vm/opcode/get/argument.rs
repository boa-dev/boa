use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `GetArgument` implements the Opcode Operation for `Opcode::GetArgument`
///
/// Operation:
///  - Get i-th argument of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetArgument;

impl GetArgument {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let value = context
            .vm
            .frame()
            .argument(index, &context.vm)
            .cloned()
            .unwrap_or_default();
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetArgument {
    const NAME: &'static str = "GetArgument";
    const INSTRUCTION: &'static str = "INST - GetArgument";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}
