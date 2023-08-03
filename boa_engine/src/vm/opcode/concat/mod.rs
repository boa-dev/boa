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
    fn operation(context: &mut Context<'_>, value_count: usize) -> JsResult<CompletionType> {
        let mut strings = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            strings.push(context.vm.pop().to_string(context)?);
        }
        strings.reverse();
        let s = JsString::concat_array(
            &strings
                .iter()
                .map(JsString::as_slice)
                .collect::<Vec<&[u16]>>(),
        );
        context.vm.push(s);
        Ok(CompletionType::Normal)
    }
}

impl Operation for ConcatToString {
    const NAME: &'static str = "ConcatToString";
    const INSTRUCTION: &'static str = "INST - ConcatToString";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value_count = context.vm.read::<u8>() as usize;
        Self::operation(context, value_count)
    }

    fn half_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value_count = context.vm.read::<u16>() as usize;
        Self::operation(context, value_count)
    }

    fn wide_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value_count = context.vm.read::<u32>() as usize;
        Self::operation(context, value_count)
    }
}
