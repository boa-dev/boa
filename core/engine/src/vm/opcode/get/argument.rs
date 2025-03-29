use crate::{
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionType, Registers,
    },
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
    #[inline(always)]
    pub(crate) fn operation(
        (index, dst): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = context
            .vm
            .frame()
            .argument(index.into(), &context.vm)
            .cloned()
            .unwrap_or_default();
        registers.set(dst.into(), value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetArgument {
    const NAME: &'static str = "GetArgument";
    const INSTRUCTION: &'static str = "INST - GetArgument";
    const COST: u8 = 2;
}
