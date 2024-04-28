use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsString,
};

/// `ConcatToString` implements the Opcode Operation for `Opcode::ConcatToString`
///
/// Operation:
///  - Concat multiple stack objects into a string.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ConcatToString;

impl ConcatToString {
    fn operation(context: &mut Context, value_count: usize) -> JsResult<CompletionType> {
        let mut strings = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            strings.push(context.vm.pop().to_string(context)?);
        }
        strings.reverse();
        let s = JsString::concat_array(
            &strings
                .iter()
                .map(JsString::as_str)
                .map(Into::into)
                .collect::<Vec<_>>(),
        );
        context.vm.push(s);
        Ok(CompletionType::Normal)
    }
}

impl Operation for ConcatToString {
    const NAME: &'static str = "ConcatToString";
    const INSTRUCTION: &'static str = "INST - ConcatToString";
    const COST: u8 = 6;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value_count = context.vm.read::<u8>() as usize;
        Self::operation(context, value_count)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let value_count = context.vm.read::<u16>() as usize;
        Self::operation(context, value_count)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let value_count = context.vm.read::<u32>() as usize;
        Self::operation(context, value_count)
    }
}
