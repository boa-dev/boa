use crate::{
    vm::{opcode::Operation, FinallyReturn, ShouldExit},
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
        let exit = context.vm.read::<u32>();
        context.vm.frame_mut().try_catch.pop();

        let finally_env = context
            .vm
            .frame_mut()
            .env_stack
            .last_mut()
            .expect("EnvStackEntries must exist");

        finally_env.set_exit_address(exit);
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
        // TODO handle a new way to get next_finally value
        let finally_candidates = context.vm.frame().env_stack.iter().filter(|env| {
            env.is_finally_env() && context.vm.frame().pc < (env.start_address() as usize)
        });

        let next_finally = match finally_candidates.last() {
            Some(env) => env.start_address(),
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
                        // handle the continuation of an abrupt break.
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
                        // Handle the continuation of an abrupt continue
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
            FinallyReturn::Err => {
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
                    } else if record.is_throw() && context.vm.frame().pc < record.target() as usize
                    {
                        context.vm.frame_mut().pc = record.target() as usize;
                        for _ in 0..context.vm.frame().env_stack.len() {
                            let env_entry = context
                                .vm
                                .frame_mut()
                                .env_stack
                                .pop()
                                .expect("EnvStackEntry must exist");

                            envs_to_pop += env_entry.env_num();
                            if env_entry.start_address() == record.target() {
                                break;
                            }
                        }
                        context.vm.frame_mut().abrupt_completion = None;
                    }

                    for _ in 0..envs_to_pop {
                        context.realm.environments.pop();
                    }
                    return Ok(ShouldExit::False);
                }
                let current_stack = context.vm.frame_mut().env_stack.pop().expect("Popping current finally stack.");

                for _ in 0..current_stack.env_num() {
                    context.realm.environments.pop();
                }
                Err(JsError::from_opaque(context.vm.pop()))
            }
        }
    }
}
