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
        let start = context.vm.frame().pc as u32 - 1;
        let exit = context.vm.read::<u32>();
        context.vm.frame_mut().try_catch.pop();

        context
            .vm
            .frame_mut()
            .env_stack
            .push(EnvStackEntry::new(start, exit).with_finally_flag());
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
        let next_finally = match context.vm.frame_mut().try_catch.last() {
            Some(addresses) if addresses.finally().is_some() => {
                addresses.finally().expect("must exist")
            }
            _ => u32::MAX,
        };

        let abrupt_record = context.vm.frame_mut().abrupt_completion;
        match context.vm.frame_mut().finally_return {
            FinallyReturn::None => {
                // Check if there is an `AbruptCompletionRecord`.
                if let Some(record) = abrupt_record {
                    let mut envs_to_pop = 0;
                    if next_finally < record.target() {
                        context.vm.frame_mut().pc = next_finally as usize;

                        for _ in 0..context.vm.frame().env_stack.len() {
                            let env_entry = context
                                .vm
                                .frame_mut()
                                .env_stack
                                .last()
                                .expect("Environment stack entries must exist");

                            if next_finally <= env_entry.exit_address() {
                                break;
                            }
                            envs_to_pop += env_entry.env_num();
                            context.vm.frame_mut().env_stack.pop();
                        }
                    } else if record.is_break() && context.vm.frame().pc < record.target() as usize
                    {
                        // Set the program counter to the target()
                        context.vm.frame_mut().pc = record.target() as usize;
                        for _ in 0..context.vm.frame().env_stack.len() {
                            let env_entry = context
                                .vm
                                .frame_mut()
                                .env_stack
                                .last()
                                .expect("Environment stack entries must exist");

                            if record.target() == env_entry.exit_address() {
                                break;
                            }
                            envs_to_pop += env_entry.env_num();
                            context.vm.frame_mut().env_stack.pop();
                        }

                        context.vm.frame_mut().abrupt_completion = None;
                    } else if record.is_continue()
                        && context.vm.frame().pc > record.target() as usize
                    {
                        context.vm.frame_mut().pc = record.target() as usize;
                        for _ in 0..context.vm.frame().env_stack.len() {
                            let env_entry = context
                                .vm
                                .frame_mut()
                                .env_stack
                                .last()
                                .expect("EnvStackEntry must exist");

                            if env_entry.start_address() == record.target() {
                                break;
                            }
                            envs_to_pop += env_entry.env_num();
                            context.vm.frame_mut().env_stack.pop();
                        }

                        context.vm.frame_mut().abrupt_completion = None;
                    }

                    for _ in 0..envs_to_pop {
                        context.realm.environments.pop();
                    }
                } else {
                    context.vm.frame_mut().env_stack.pop();
                }

                Ok(ShouldExit::False)
            }
            FinallyReturn::Ok => Ok(ShouldExit::True),
            FinallyReturn::Err => Err(JsError::from_opaque(context.vm.pop())),
        }
    }
}
