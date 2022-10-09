use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PushLiteral;

impl Operation for PushLiteral {
    const NAME: &'static str = "PushLiteral";
    const INSTRUCTION: &'static str = "INST - PushLiteral";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>() as usize;
        let value = context.vm.frame().code.literals[index].clone();
        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}
