use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsString,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ConcatToString;

impl Operation for ConcatToString {
    const NAME: &'static str = "ConcatToString";
    const INSTRUCTION: &'static str = "INST - ConcatToString";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value_count = context.vm.read::<u32>();
        let mut strings = Vec::with_capacity(value_count as usize);
        for _ in 0..value_count {
            strings.push(context.vm.pop().to_string(context)?);
        }
        strings.reverse();
        let s =
            JsString::concat_array(&strings.iter().map(JsString::as_str).collect::<Vec<&str>>());
        context.vm.push(s);
        Ok(ShouldExit::False)
    }
}
