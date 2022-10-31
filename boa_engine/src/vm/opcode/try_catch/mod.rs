use crate::{
    vm::{opcode::Operation, CatchAddresses, FinallyReturn, ShouldExit, TryStackEntry},
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
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
        context.vm.frame_mut().try_env_stack.push(TryStackEntry {
            num_env: 0,
            num_loop_stack_entries: 0,
        });
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        context.vm.frame_mut().catch.pop();
        let try_stack_entry = context
            .vm
            .frame_mut()
            .try_env_stack
            .pop()
            .expect("must exist");
        for _ in 0..try_stack_entry.num_env {
            context.realm.environments.pop();
        }
        let mut num_env = try_stack_entry.num_env;
        for _ in 0..try_stack_entry.num_loop_stack_entries {
            num_env -= context
                .vm
                .frame_mut()
                .loop_env_stack
                .pop()
                .expect("must exist");
        }
        *context
            .vm
            .frame_mut()
            .loop_env_stack
            .last_mut()
            .expect("must exist") -= num_env;
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let finally = context.vm.read::<u32>();
        context.vm.frame_mut().catch.push(CatchAddresses {
            next: finally,
            finally: Some(finally),
        });
        context.vm.frame_mut().try_env_stack.push(TryStackEntry {
            num_env: 0,
            num_loop_stack_entries: 0,
        });
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        context.vm.frame_mut().catch.pop();
        let try_stack_entry = context
            .vm
            .frame_mut()
            .try_env_stack
            .pop()
            .expect("must exist");
        for _ in 0..try_stack_entry.num_env {
            context.realm.environments.pop();
        }
        let mut num_env = try_stack_entry.num_env;
        for _ in 0..try_stack_entry.num_loop_stack_entries {
            num_env -= context
                .vm
                .frame_mut()
                .loop_env_stack
                .pop()
                .expect("must exist");
        }
        *context
            .vm
            .frame_mut()
            .loop_env_stack
            .last_mut()
            .expect("must exist") -= num_env;
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let frame = context.vm.frame_mut();
        if frame.finally_return == FinallyReturn::Err {
            frame.finally_return = FinallyReturn::None;
        }
        Ok(ShouldExit::False)
    }
}
