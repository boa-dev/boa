use crate::{
    vm::{call_frame::AbruptCompletionRecord, opcode::Operation, FinallyReturn, ShouldExit},
    Context, JsResult,
};

/// `Throw` implements the Opcode Operation for `Opcode::Throw`
///
/// Operation:
///  - Throw exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Throw;

impl Operation for Throw {
    const NAME: &'static str = "Throw";
    const INSTRUCTION: &'static str = "INST - Throw";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        // 1. Find the next active catch block
        let viable_catch_candidates = context
            .vm
            .frame()
            .env_stack
            .iter()
            .filter(|env| env.is_try_env() && env.start_address() < env.exit_address());
        let current_address = context.vm.frame().pc;
        //println!("{:#?}", viable_catch_candidates);

        if let Some(candidate) = viable_catch_candidates.last() {
            let catch_target = candidate.start_address();

            let mut env_to_pop = 0;
            let mut target_address = u32::MAX;
            while context.vm.frame().env_stack.len() > 1 {
                let env_entry = context
                    .vm
                    .frame_mut()
                    .env_stack
                    .last()
                    .expect("EnvStackEntries must exist");

                if env_entry.is_try_env() && env_entry.start_address() < env_entry.exit_address() {
                    target_address = env_entry.start_address();
                    env_to_pop += env_entry.env_num();
                    context.vm.frame_mut().env_stack.pop();
                    break;
                } else if env_entry.is_finally_env() {
                    if current_address > env_entry.start_address() as usize {
                        target_address = env_entry.exit_address();
                    } else {
                        target_address = env_entry.start_address();
                    }

                    break;
                }
                env_to_pop += env_entry.env_num();
                context.vm.frame_mut().env_stack.pop();
            }

            for _ in 0..env_to_pop {
                context.realm.environments.pop();
            }

            if target_address != catch_target {
                context.vm.frame_mut().pc = target_address as usize;
                let record = AbruptCompletionRecord::default()
                    .with_throw_flag()
                    .with_initial_target(catch_target);
                context.vm.frame_mut().abrupt_completion = Some(record);
            } else {
                context.vm.frame_mut().pc = target_address as usize;
            }

            context.vm.frame_mut().finally_return = FinallyReturn::None;
            return Ok(ShouldExit::False);
        }

        let mut env_to_pop = 0;
        let mut target_address = None;
        while !context.vm.frame().env_stack.is_empty() {
            let env_entry = context
                .vm
                .frame()
                .env_stack
                .last()
                .expect("EnvStackEntry must exist");

            if env_entry.is_finally_env() {
                if (env_entry.start_address() as usize) < current_address {
                    target_address = Some(env_entry.exit_address() as usize);
                } else {
                    target_address = Some(env_entry.start_address() as usize);
                }
                break;
            }

            env_to_pop += env_entry.env_num();
            if env_entry.is_global_env() {
                break;
            }

            context.vm.frame_mut().env_stack.pop();
        }

        for _ in 0..env_to_pop {
            context.realm.environments.pop();
        }

        if let Some(address) = target_address {
            context.vm.frame_mut().pc = address;
            context.vm.frame_mut().finally_return = FinallyReturn::None;
            return Ok(ShouldExit::False);
        }

        context.vm.pop();
        Ok(ShouldExit::True)
    }
}
