use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `Pop` implements the Opcode Operation for `Opcode::Pop`
///
/// Operation:
///  - Pop the top value from the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Pop;

impl Operation for Pop {
    const NAME: &'static str = "Pop";
    const INSTRUCTION: &'static str = "INST - Pop";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let _val = context.vm.pop();
        Ok(CompletionType::Normal)
    }
}

/// `PopEnvironment` implements the Opcode Operation for `Opcode::PopEnvironment`
///
/// Operation:
///  - Pop the current environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopEnvironment;

impl Operation for PopEnvironment {
    const NAME: &'static str = "PopEnvironment";
    const INSTRUCTION: &'static str = "INST - PopEnvironment";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        context.vm.environments.pop();
        context.vm.frame_mut().dec_frame_env_stack();
        Ok(CompletionType::Normal)
    }
}
