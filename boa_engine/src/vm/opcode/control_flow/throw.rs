use crate::{
    js_string,
    vm::{call_frame::AbruptCompletionRecord, opcode::Operation, CompletionType},
    Context, JsError, JsNativeError, JsResult,
};
use thin_vec::ThinVec;

/// `Throw` implements the Opcode Operation for `Opcode::Throw`
///
/// Operation:
///  - Throw exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Throw;

impl Operation for Throw {
    const NAME: &'static str = "Throw";
    const INSTRUCTION: &'static str = "INST - Throw";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let error = if let Some(err) = context.vm.err.take() {
            err
        } else {
            JsError::from_opaque(context.vm.pop())
        };

        // Close all iterators that are still open.
        let mut iterators = ThinVec::new();
        std::mem::swap(&mut iterators, &mut context.vm.frame_mut().iterators);
        for (iterator, done) in iterators {
            if done {
                continue;
            }
            if let Ok(Some(f)) = iterator.get_method(js_string!("return"), context) {
                drop(f.call(&iterator.into(), &[], context));
            }
        }
        context.vm.err.take();

        // 1. Find the viable catch and finally blocks
        let current_address = context.vm.frame().pc;
        let viable_catch_candidates = context
            .vm
            .frame()
            .env_stack
            .iter()
            .filter(|env| env.is_try_env() && env.start_address() < env.exit_address());

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
                    if current_address > (env_entry.start_address() as usize) {
                        target_address = env_entry.exit_address();
                    } else {
                        target_address = env_entry.start_address();
                    }
                    break;
                }
                env_to_pop += env_entry.env_num();
                context.vm.frame_mut().env_stack.pop();
            }

            let env_truncation_len = context.vm.environments.len().saturating_sub(env_to_pop);
            context.vm.environments.truncate(env_truncation_len);

            if target_address == catch_target {
                context.vm.frame_mut().pc = catch_target as usize;
            } else {
                context.vm.frame_mut().pc = target_address as usize;
            };

            for _ in 0..context.vm.frame().pop_on_return {
                context.vm.pop();
            }

            context.vm.frame_mut().pop_on_return = 0;
            let record = AbruptCompletionRecord::new_throw().with_initial_target(catch_target);
            context.vm.frame_mut().abrupt_completion = Some(record);
            let err = error.to_opaque(context);
            context.vm.push(err);
            return Ok(CompletionType::Normal);
        }

        let mut env_to_pop = 0;
        let mut target_address = None;
        let mut env_stack_to_pop = 0;
        for env_entry in context.vm.frame_mut().env_stack.iter_mut().rev() {
            if env_entry.is_finally_env() {
                if (env_entry.start_address() as usize) < current_address {
                    target_address = Some(env_entry.exit_address() as usize);
                } else {
                    target_address = Some(env_entry.start_address() as usize);
                }
                break;
            };

            env_to_pop += env_entry.env_num();
            if env_entry.is_global_env() {
                env_entry.clear_env_num();
                break;
            };

            env_stack_to_pop += 1;
        }

        let record = AbruptCompletionRecord::new_throw();
        context.vm.frame_mut().abrupt_completion = Some(record);

        if let Some(address) = target_address {
            for _ in 0..env_stack_to_pop {
                context.vm.frame_mut().env_stack.pop();
            }

            let env_truncation_len = context.vm.environments.len().saturating_sub(env_to_pop);
            context.vm.environments.truncate(env_truncation_len);

            let previous_stack_size = context
                .vm
                .stack
                .len()
                .saturating_sub(context.vm.frame().pop_on_return);
            context.vm.stack.truncate(previous_stack_size);
            context.vm.frame_mut().pop_on_return = 0;

            context.vm.frame_mut().pc = address;
            let err = error.to_opaque(context);
            context.vm.push(err);
            return Ok(CompletionType::Normal);
        }

        context.vm.err = Some(error);
        Ok(CompletionType::Throw)
    }
}

/// `ThrowNewTypeError` implements the Opcode Operation for `Opcode::ThrowNewTypeError`
///
/// Operation:
///  - Throws a `TypeError` exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowNewTypeError;

impl Operation for ThrowNewTypeError {
    const NAME: &'static str = "ThrowNewTypeError";
    const INSTRUCTION: &'static str = "INST - ThrowNewTypeError";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let msg = context.vm.frame().code_block.literals[index as usize]
            .as_string()
            .expect("throw message must be a string")
            .clone();
        let msg = msg
            .to_std_string()
            .expect("throw message must be an ASCII string");
        Err(JsNativeError::typ().with_message(msg).into())
    }
}
