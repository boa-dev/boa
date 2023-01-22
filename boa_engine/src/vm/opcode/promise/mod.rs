use crate::{
    vm::{call_frame::EnvStackEntry, opcode::Operation, FinallyReturn, ShouldExit},
    Context, JsError, JsResult,
};

/// `FinallyStart` implements the Opcode Operation for `Opcode::FinallyStart`
///
/// Operation:
///  - Start of a finally block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FinallyStart;

impl Operation for FinallyStart {
    const NAME: &'static str = "FinallyStart";
    const INSTRUCTION: &'static str = "INST - FinallyStart";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        context.vm.frame_mut().finally_jump.pop();

        context
            .vm
            .frame_mut()
            .env_stack
            .push(EnvStackEntry::default().with_finally_flag());
        Ok(ShouldExit::False)
    }
}

/// `FinallyEnd` implements the Opcode Operation for `Opcode::FinallyEnd`
///
/// Operation:
///  - End of a finally block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FinallyEnd;

impl Operation for FinallyEnd {
    const NAME: &'static str = "FinallyEnd";
    const INSTRUCTION: &'static str = "INST - FinallyEnd";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let next_finally = match context.vm.frame_mut().finally_jump.last() {
            Some(address) if address.is_some() => address.expect("must exist"),
            _ => u32::MAX,
        };

        let abrupt_record = context.vm.frame_mut().abrupt_completion;
        match context.vm.frame_mut().finally_return {
            FinallyReturn::None => {
                // Check if there is an `AbruptCompletionRecord`.
                if let Some(record) = abrupt_record {
                    if next_finally < record.target() {
                        context.vm.frame_mut().pc = next_finally as usize;
                        context.realm.environments.pop();
                        context.vm.frame_mut().dec_frame_env_stack();
                    } else if record.is_break() && context.vm.frame().pc < record.target() as usize
                    {
                        context.vm.frame_mut().pc = record.target() as usize;
                        let envs_to_pop = record.envs();
                        for _ in 0..envs_to_pop {
                            context.realm.environments.pop();
                            context.vm.frame_mut().dec_frame_env_stack();
                        }
                        context.vm.frame_mut().abrupt_completion = None;
                    }
                }

                Ok(ShouldExit::False)
            }
            FinallyReturn::Ok => Ok(ShouldExit::True),
            FinallyReturn::Err => Err(JsError::from_opaque(context.vm.pop())),
        }
    }
}

/// `FinallySetJump` implements the Opcode Operation for `Opcode::FinallySetJump`
///
/// Operation:
///  - Set the address for a finally jump.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FinallySetJump;

impl Operation for FinallySetJump {
    const NAME: &'static str = "FinallySetJump";
    const INSTRUCTION: &'static str = "INST - FinallySetJump";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        *context
            .vm
            .frame_mut()
            .finally_jump
            .last_mut()
            .expect("finally jump must exist here") = Some(address);
        Ok(ShouldExit::False)
    }
}
