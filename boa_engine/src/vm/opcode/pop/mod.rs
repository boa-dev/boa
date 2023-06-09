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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let _val = context.vm.pop();
        Ok(CompletionType::Normal)
    }
}

/// `PopIfThrown` implements the Opcode Operation for `Opcode::PopIfThrown`
///
/// Operation:
///  - Pop the top value from the stack if the last try block has thrown a value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopIfThrown;

impl Operation for PopIfThrown {
    const NAME: &'static str = "PopIfThrown";
    const INSTRUCTION: &'static str = "INST - PopIfThrown";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let frame = context.vm.frame();
        match frame.abrupt_completion {
            Some(record) if record.is_throw() => {
                context.vm.pop();
            }
            _ => {}
        };
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        context.vm.environments.pop();
        context.vm.frame_mut().dec_frame_env_stack();
        Ok(CompletionType::Normal)
    }
}

/// `PopReturnAdd` implements the Opcode Operation for `Opcode::PopReturnAdd`
///
/// Operation:
///  - Add one to the pop on return count.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopOnReturnAdd;

impl Operation for PopOnReturnAdd {
    const NAME: &'static str = "PopOnReturnAdd";
    const INSTRUCTION: &'static str = "INST - PopOnReturnAdd";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        context.vm.frame_mut().pop_on_return += 1;
        Ok(CompletionType::Normal)
    }
}

/// `PopOnReturnSub` implements the Opcode Operation for `Opcode::PopOnReturnSub`
///
/// Operation:
///  - Subtract one from the pop on return count.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopOnReturnSub;

impl Operation for PopOnReturnSub {
    const NAME: &'static str = "PopOnReturnSub";
    const INSTRUCTION: &'static str = "INST - PopOnReturnSub";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        context.vm.frame_mut().pop_on_return -= 1;
        Ok(CompletionType::Normal)
    }
}
