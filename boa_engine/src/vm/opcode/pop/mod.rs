use crate::{
    vm::{opcode::Operation, ShouldExit},
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let _val = context.vm.pop();
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let frame = context.vm.frame_mut();
        if frame.thrown {
            frame.thrown = false;
            context.vm.pop();
        }
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        context.realm.environments.pop();
        context.vm.frame_mut().loop_env_stack_dec();
        context.vm.frame_mut().try_env_stack_dec();
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        context.vm.frame_mut().pop_on_return += 1;
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        context.vm.frame_mut().pop_on_return -= 1;
        Ok(ShouldExit::False)
    }
}
