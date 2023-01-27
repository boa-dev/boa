use crate::{
    vm::{call_frame::EnvStackEntry, opcode::Operation, ShouldExit},
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

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let start = context.vm.frame().pc as u32 - 1;
        let exit = context.vm.read::<u32>();

        context
            .vm
            .frame_mut()
            .env_stack
            .push(EnvStackEntry::new(start, exit).with_loop_flag());
        Ok(ShouldExit::False)
    }
}

/// `LoopContinue` implements the Opcode Operation for `Opcode::LoopContinue`.
///
/// Operation:
///  - Pushes a clean environment onto the frame's `EnvEntryStack`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopContinue;

impl Operation for LoopContinue {
    const NAME: &'static str = "LoopContinue";
    const INSTRUCTION: &'static str = "INST - LoopContinue";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let start = context.vm.read::<u32>();
        let exit = context.vm.read::<u32>();

        // 1. Clean up the previous environment.
        if context
            .vm
            .frame_mut()
            .env_stack
            .last()
            .expect("this must exist")
            .exit_address()
            == exit
        {
            let popped_entry = context
                .vm
                .frame_mut()
                .env_stack
                .pop()
                .expect("EnvStackEntries must exist");

            for _ in 0..popped_entry.env_num() {
                context.realm.environments.pop();
            }
        }

        // 2. Push a new clean EnvStack.
        context
            .vm
            .frame_mut()
            .env_stack
            .push(EnvStackEntry::new(start, exit).with_loop_flag());

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

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let mut envs_to_pop = 0_usize;
        for _ in 1..context.vm.frame().env_stack.len() {
            let env_entry = context
                .vm
                .frame_mut()
                .env_stack
                .pop()
                .expect("this must exist");
            envs_to_pop += env_entry.env_num();

            if env_entry.is_loop_env() {
                break;
            }
        }

        for _ in 0..envs_to_pop {
            context.realm.environments.pop();
        }
        Ok(ShouldExit::False)
    }
}
