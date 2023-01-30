use crate::{
    vm::{call_frame::EnvStackEntry, opcode::Operation, FinallyReturn, ShouldExit},
    Context, JsResult,
};

/// `CatchStart` implements the Opcode Operation for `Opcode::CatchStart`
///
/// Operation:
///  - Start of a catch block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CatchStart;

impl Operation for CatchStart {
    const NAME: &'static str = "CatchStart";
    const INSTRUCTION: &'static str = "INST - CatchStart";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let start = context.vm.frame().pc as u32 - 1;
        let finally = context.vm.read::<u32>();

        context
            .vm
            .frame_mut()
            .env_stack
            .push(EnvStackEntry::new(start, finally).with_catch_flag());
        context.vm.frame_mut().thrown = false;
        Ok(ShouldExit::False)
    }
}

/// `CatchEnd` implements the Opcode Operation for `Opcode::CatchEnd`
///
/// Operation:
///  - End of a catch block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CatchEnd;

impl Operation for CatchEnd {
    const NAME: &'static str = "CatchEnd";
    const INSTRUCTION: &'static str = "INST - CatchEnd";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        context.vm.frame_mut().try_catch.pop();
        let mut envs_to_pop = 0_usize;
        for _ in 1..context.vm.frame().env_stack.len() {
            let env_entry = context
                .vm
                .frame_mut()
                .env_stack
                .pop()
                .expect("this must exist");

            envs_to_pop += env_entry.env_num();
            if env_entry.is_catch_env() {
                break;
            }
        }

        for _ in 0..envs_to_pop {
            context.realm.environments.pop();
        }
        context.vm.frame_mut().finally_return = FinallyReturn::None;
        Ok(ShouldExit::False)
    }
}

/// `CatchEnd2` implements the Opcode Operation for `Opcode::CatchEnd2`
///
/// Operation:
///  - End of a catch block
#[derive(Debug, Clone, Copy)]
pub(crate) struct CatchEnd2;

impl Operation for CatchEnd2 {
    const NAME: &'static str = "CatchEnd2";
    const INSTRUCTION: &'static str = "INST - CatchEnd2";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let frame = context.vm.frame_mut();
        if frame
            .env_stack
            .last()
            .expect("Env stack entry must exist")
            .is_catch_env()
        {
            let catch_entry = frame
                .env_stack
                .pop()
                .expect("must exist as catch env entry");

            for _ in 0..catch_entry.env_num() {
                context.realm.environments.pop();
            }
        }

        if frame.finally_return == FinallyReturn::Err {
            frame.finally_return = FinallyReturn::None;
        }
        Ok(ShouldExit::False)
    }
}
