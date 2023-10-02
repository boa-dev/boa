use crate::{
    object::JsRegExp,
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
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `PushRegExp` implements the Opcode Operation for `Opcode::PushRegExp`
///
/// Operation:
///  - Push regexp value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushRegExp;

impl PushRegExp {
    fn operation(
        context: &mut Context<'_>,
        pattern_index: usize,
        flags_index: usize,
    ) -> JsResult<CompletionType> {
        let pattern = context.vm.frame().code_block.names[pattern_index].clone();
        let flags = context.vm.frame().code_block.names[flags_index].clone();

        let regexp = JsRegExp::new(pattern, flags, context)?;
        context.vm.push(regexp);
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushRegExp {
    const NAME: &'static str = "PushRegExp";
    const INSTRUCTION: &'static str = "INST - PushRegExp";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let pattern_index = context.vm.read::<u8>() as usize;
        let flags_index = context.vm.read::<u8>() as usize;
        Self::operation(context, pattern_index, flags_index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let pattern_index = context.vm.read::<u16>() as usize;
        let flags_index = context.vm.read::<u16>() as usize;
        Self::operation(context, pattern_index, flags_index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let pattern_index = context.vm.read::<u32>() as usize;
        let flags_index = context.vm.read::<u32>() as usize;
        Self::operation(context, pattern_index, flags_index)
    }
}
