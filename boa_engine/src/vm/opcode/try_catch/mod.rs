use crate::{
    vm::{opcode::Operation, CatchAddresses, FinallyReturn, ShouldExit, call_frame::EnvStackEntry},
    Context, JsResult,
};

/// `TryStart` implements the Opcode Operation for `Opcode::TryStart`
///
/// Operation:
///  - Start of a try block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TryStart;

impl Operation for TryStart {
    const NAME: &'static str = "TryStart";
    const INSTRUCTION: &'static str = "INST - TryStart";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let next = context.vm.read::<u32>();
        let finally = context.vm.read::<u32>();
        let finally = if finally == 0 { None } else { Some(finally) };
        context
            .vm
            .frame_mut()
            .catch
            .push(CatchAddresses { next, finally });
        context.vm.frame_mut().finally_jump.push(None);
        context.vm.frame_mut().finally_return = FinallyReturn::None;
        context.vm.frame_mut().env_stack.push(EnvStackEntry::default().with_try_flag());
        Ok(ShouldExit::False)
    }
}

/// `TryEnd` implements the Opcode Operation for `Opcode::TryEnd`
///
/// Operation:
///  - End of a try block
#[derive(Debug, Clone, Copy)]
pub(crate) struct TryEnd;

impl Operation for TryEnd {
    const NAME: &'static str = "TryEnd";
    const INSTRUCTION: &'static str = "INST - TryEnd";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        context.vm.frame_mut().catch.pop();
        let mut envs_to_pop = 0_usize;
        for _ in 1..context.vm.frame().env_stack.len() {
            let env_entry = context.vm.frame_mut().env_stack.pop().expect("this must exist");
            envs_to_pop += env_entry.env_num();

            if env_entry.is_try_env() {
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
        let finally = context.vm.read::<u32>();
        context.vm.frame_mut().catch.push(CatchAddresses {
            next: finally,
            finally: Some(finally),
        });
        context.vm.frame_mut().env_stack.push(EnvStackEntry::default().with_try_flag());
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
        context.vm.frame_mut().catch.pop();
        let mut envs_to_pop = 0_usize;
        for _ in 1..context.vm.frame().env_stack.len() {
            let env_entry = context.vm.frame_mut().env_stack.pop().expect("this must exist");
            envs_to_pop += env_entry.env_num();

            if env_entry.is_try_env() {
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
        if frame.finally_return == FinallyReturn::Err {
            frame.finally_return = FinallyReturn::None;
        }
        Ok(ShouldExit::False)
    }
}
