use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `LoopStart` implements the Opcode Operation for `Opcode::LoopStart`
///
/// Operation:
///  - Push loop start marker.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopStart;

impl Operation for LoopStart {
    const NAME: &'static str = "LoopStart";
    const INSTRUCTION: &'static str = "INST - LoopStart";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        context.vm.frame_mut().loop_env_stack.push(0);
        context.vm.frame_mut().try_env_stack_loop_inc();
        Ok(ShouldExit::False)
    }
}

/// `LoopContinue` implements the Opcode Operation for `Opcode::LoopContinue`
///
/// Operation:
///  - Clean up environments when a loop continues.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopContinue;

impl Operation for LoopContinue {
    const NAME: &'static str = "LoopContinue";
    const INSTRUCTION: &'static str = "INST - LoopContinue";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let env_num = context
            .vm
            .frame_mut()
            .loop_env_stack
            .last_mut()
            .expect("loop env stack entry must exist");
        let env_num_copy = *env_num;
        *env_num = 0;
        for _ in 0..env_num_copy {
            context.realm.environments.pop();
        }
        Ok(ShouldExit::False)
    }
}

/// `LoopEnd` implements the Opcode Operation for `Opcode::LoopEnd`
///
/// Operation:
///  - Clean up enviroments at the end of a lopp.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopEnd;

impl Operation for LoopEnd {
    const NAME: &'static str = "LoopEnd";
    const INSTRUCTION: &'static str = "INST - LoopEnd";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let env_num = context
            .vm
            .frame_mut()
            .loop_env_stack
            .pop()
            .expect("loop env stack entry must exist");
        for _ in 0..env_num {
            context.realm.environments.pop();
            context.vm.frame_mut().try_env_stack_dec();
        }
        context.vm.frame_mut().try_env_stack_loop_dec();
        Ok(ShouldExit::False)
    }
}
