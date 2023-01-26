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
        let jump_address = context.vm.read::<u32>();
        let target_address = context.vm.read::<u32>();

        // 1. Iterate through Env stack looking for exit address.
        let mut envs_to_pop = 0;
        for _ in 0..context.vm.frame().env_stack.len() {
            let env_entry = context
                .vm
                .frame_mut()
                .env_stack
                .last()
                .expect("EnvStackEntry must exist");

            if jump_address <= env_entry.exit_address() {
                break;
            }
            envs_to_pop += env_entry.env_num();
            context.vm.frame_mut().env_stack.pop();
        }

        for _ in 0..envs_to_pop {
            context.realm.environments.pop();
        }

        // 2. Register target address in AbruptCompletionRecord.
        if jump_address < target_address {
            let new_record = AbruptCompletionRecord::default()
                .with_break_flag()
                .with_initial_target(target_address);
            context.vm.frame_mut().abrupt_completion = Some(new_record);
        }

        // 3. Set program counter and finally return fields.
        context.vm.frame_mut().pc = jump_address as usize;
        context.vm.frame_mut().finally_return = FinallyReturn::None;
        Ok(ShouldExit::False)
    }
}

/// `Break` implements the Opcode Operation for `Opcode::Break`
///
/// Operation:
///   - Pop required environments and jump to address.
pub(crate) struct OldBreak;

impl Operation for OldBreak {
    const NAME: &'static str = "Break";
    const INSTRUCTION: &'static str = "INST - Break";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        let pop_envs = context.vm.read::<u32>();

        for _ in 0..pop_envs {
            context.realm.environments.pop();

            context.vm.frame_mut().dec_frame_env_stack();
        }

        context.vm.frame_mut().pc = address as usize;
        context.vm.frame_mut().finally_return = FinallyReturn::None;
        Ok(ShouldExit::False)
    }
}

/// `Continue` implements the Opcode Operation for `Opcode::Continue`
///
/// Operands:
///   - Target address
///   - Initial environments to reconcile on continue (will be tracked along with changes to environment stack)
///
/// Operation:
///   - Initializes the `AbruptCompletionRecord` for a delayed continued in a `Opcode::FinallyEnd`
pub(crate) struct Continue;

impl Operation for Continue {
    const NAME: &'static str = "Continue";
    const INSTRUCTION: &'static str = "INST - Continue";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let jump_address = context.vm.read::<u32>();
        let target_address = context.vm.read::<u32>();

        // 1. Iterate through Env stack looking for exit address.
        let mut envs_to_pop = 0;
        for _ in 0..context.vm.frame().env_stack.len() {
            let env_entry = context
                .vm
                .frame_mut()
                .env_stack
                .last()
                .expect("EnvStackEntry must exist");

            if env_entry.start_address() <= jump_address {
                break;
            }

            envs_to_pop += env_entry.env_num();
            context.vm.frame_mut().env_stack.pop();
        }

        for _ in 0..envs_to_pop {
            context.realm.environments.pop();
        }

        // 2. Register target address in AbruptCompletionRecord.
        if jump_address > target_address {
            let new_record = AbruptCompletionRecord::default()
                .with_continue_flag()
                .with_initial_target(target_address);
            context.vm.frame_mut().abrupt_completion = Some(new_record);
        }

        // 3. Set program counter and finally return fields.
        context.vm.frame_mut().pc = jump_address as usize;
        context.vm.frame_mut().finally_return = FinallyReturn::None;
        Ok(ShouldExit::False)
    }
}
