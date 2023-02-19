use crate::{
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context, JsString,
};

/// `ConcatToString` implements the Opcode Operation for `Opcode::ConcatToString`
///
/// Operation:
///  - Concat multiple stack objects into a string.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ConcatToString;

impl Operation for ConcatToString {
    const NAME: &'static str = "ConcatToString";
    const INSTRUCTION: &'static str = "INST - ConcatToString";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value_count = context.vm.read::<u32>();
        let mut strings = Vec::with_capacity(value_count as usize);
        for _ in 0..value_count {
            strings.push(ok_or_throw_completion!(
                context.vm.pop().to_string(context),
                context
            ));
        }
        strings.reverse();
        let s = JsString::concat_array(
            &strings
                .iter()
                .map(JsString::as_slice)
                .collect::<Vec<&[u16]>>(),
        );
        context.vm.push(s);
        CompletionType::Normal
    }
}
