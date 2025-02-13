use crate::{
    object::JsRegExp,
    vm::{opcode::Operation, CompletionType, Constant, Registers},
    Context, JsResult, JsValue,
};

/// `PushLiteral` implements the Opcode Operation for `Opcode::PushLiteral`
///
/// Operation:
///  - Push literal value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushLiteral;

impl PushLiteral {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        dst: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let constant = &context.vm.frame().code_block().constants[index];
        let value: JsValue = match constant {
            Constant::BigInt(v) => v.clone().into(),
            Constant::String(v) => v.clone().into(),
            _ => unreachable!("constant should be a string or bigint"),
        };
        registers.set(dst, value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushLiteral {
    const NAME: &'static str = "PushLiteral";
    const INSTRUCTION: &'static str = "INST - PushLiteral";
    const COST: u8 = 1;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(dst, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(dst, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(dst, index, registers, context)
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
        dst: u32,
        pattern_index: usize,
        flags_index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let code_block = context.vm.frame().code_block();
        let pattern = code_block.constant_string(pattern_index);
        let flags = code_block.constant_string(flags_index);
        let regexp = JsRegExp::new(pattern, flags, context)?;
        registers.set(dst, regexp.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushRegExp {
    const NAME: &'static str = "PushRegExp";
    const INSTRUCTION: &'static str = "INST - PushRegExp";
    const COST: u8 = 5;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u8>().into();
        let pattern_index = context.vm.read::<u8>() as usize;
        let flags_index = context.vm.read::<u8>() as usize;
        Self::operation(dst, pattern_index, flags_index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u16>().into();
        let pattern_index = context.vm.read::<u16>() as usize;
        let flags_index = context.vm.read::<u16>() as usize;
        Self::operation(dst, pattern_index, flags_index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        let pattern_index = context.vm.read::<u32>() as usize;
        let flags_index = context.vm.read::<u32>() as usize;
        Self::operation(dst, pattern_index, flags_index, registers, context)
    }
}
