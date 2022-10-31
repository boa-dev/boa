use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `LogicalNot` implements the Opcode Operation for `Opcode::LogicalNot`
///
/// Operation:
///  - Unary logical `!` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogicalNot;

impl Operation for LogicalNot {
    const NAME: &'static str = "LogicalNot";
    const INSTRUCTION: &'static str = "INST - LogicalNot";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        context.vm.push(!value.to_boolean());
        Ok(ShouldExit::False)
    }
}
