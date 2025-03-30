use crate::{
    object::JsRegExp,
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionType, Constant, Registers,
    },
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
    #[inline(always)]
    pub(crate) fn operation(
        (dst, index): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let constant = &context.vm.frame().code_block().constants[usize::from(index)];
        let value: JsValue = match constant {
            Constant::BigInt(v) => v.clone().into(),
            Constant::String(v) => v.clone().into(),
            _ => unreachable!("constant should be a string or bigint"),
        };
        registers.set(dst.into(), value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushLiteral {
    const NAME: &'static str = "PushLiteral";
    const INSTRUCTION: &'static str = "INST - PushLiteral";
    const COST: u8 = 1;
}

/// `PushRegexp` implements the Opcode Operation for `Opcode::PushRegexp`
///
/// Operation:
///  - Push regexp value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushRegexp;

impl PushRegexp {
    #[inline(always)]
    pub(crate) fn operation(
        (dst, pattern_index, flags_index): (VaryingOperand, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let code_block = context.vm.frame().code_block();
        let pattern = code_block.constant_string(pattern_index.into());
        let flags = code_block.constant_string(flags_index.into());
        let regexp = JsRegExp::new(pattern, flags, context)?;
        registers.set(dst.into(), regexp.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushRegexp {
    const NAME: &'static str = "PushRegexp";
    const INSTRUCTION: &'static str = "INST - PushRegexp";
    const COST: u8 = 5;
}
