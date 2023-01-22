use crate::{
    vm::{call_frame::AbruptCompletionRecord, opcode::Operation, FinallyReturn, ShouldExit},
    Context, JsResult,
};

/// `Break` implements the Opcode Operation for `Opcode::Break`
///
/// Operation:
///   - Pop required environments and jump to address.
pub(crate) struct Break;

impl Operation for Break {
    const NAME: &'static str = "Break";
    const INSTRUCTION: &'static str = "INST - Break";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        let pop_envs = context.vm.read::<u32>();

        for _ in 0..pop_envs {
            context.realm.environments.pop();

            context.vm.frame_mut().dec_frame_env_stack();

            if context
                .vm
                .frame()
                .env_stack
                .last()
                .expect("must exist")
                .env_num()
                == 0
            {
                context.vm.frame_mut().env_stack.pop();
            }
        }

        context.vm.frame_mut().pc = address as usize;
        context.vm.frame_mut().finally_return = FinallyReturn::None;
        Ok(ShouldExit::False)
    }
}

/// `SetBreakTarget` implements the Opcode Operation for `Opcode::SetBreakTarget`
///
/// Operands:
///   - Target address
///   - Initial environments to reconcile on break (will be tracked along with changes to environment stack)
///
/// Operation:
///   - Initializes the `AbruptCompletionRecord` for a delayed break in a `Opcode::FinallyEnd`
pub(crate) struct SetBreakTarget;

impl Operation for SetBreakTarget {
    const NAME: &'static str = "SetBreakTarget";
    const INSTRUCTION: &'static str = "INST - SetBreakTarget";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        let pop_envs = context.vm.read::<u32>();

        if let Some(record) = context.vm.frame_mut().abrupt_completion.as_mut() {
            record.set_break_flag();
            record.set_target(address);
            record.set_envs(pop_envs);
        } else {
            let new_record = AbruptCompletionRecord::default()
                .with_break_flag()
                .with_initial_target(address)
                .with_initial_envs(pop_envs);
            context.vm.frame_mut().abrupt_completion = Some(new_record);
        }

        Ok(ShouldExit::False)
    }
}

/// `SetContinueTarget` implements the Opcode Operation for `Opcode::SetContinueTarget`
///
/// Operands:
///   - Target address
///   - Initial environments to reconcile on continue (will be tracked along with changes to environment stack)
///
/// Operation:
///   - Initializes the `AbruptCompletionRecord` for a delayed continued in a `Opcode::FinallyEnd`
pub(crate) struct SetContinueTarget;

impl Operation for SetContinueTarget {
    const NAME: &'static str = "SetContinueTarget";
    const INSTRUCTION: &'static str = "INST - SetContinueTarget";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        let pop_envs = context.vm.read::<u32>();

        if let Some(record) = context.vm.frame_mut().abrupt_completion.as_mut() {
            record.set_continue_flag();
            record.set_target(address);
            record.set_envs(pop_envs);
        } else {
            let new_record = AbruptCompletionRecord::default()
                .with_continue_flag()
                .with_initial_target(address)
                .with_initial_envs(pop_envs);
            context.vm.frame_mut().abrupt_completion = Some(new_record);
        }

        Ok(ShouldExit::False)
    }
}
