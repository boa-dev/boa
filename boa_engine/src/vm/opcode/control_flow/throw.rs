use crate::{
    vm::{
        call_frame::{AbruptCompletionRecord, EnvStackEntry},
        opcode::Operation,
        CompletionType,
    },
    Context, JsError, JsNativeError, JsResult, JsValue,
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let error = if let Some(err) = context.vm.err.take() {
            err
        } else {
            JsError::from_opaque(context.vm.pop())
        };

        // 1. Find the viable catch and finally blocks
        let current_address = context.vm.frame().pc;
        let mut envs = context.vm.frame().env_stack.iter();

        if let Some(idx) =
            envs.rposition(|env| env.is_try_env() && env.start_address() < env.exit_address())
        {
            let active_iterator = context.vm.frame().env_stack[..idx]
                .iter()
                .filter_map(EnvStackEntry::iterator)
                .last();

            // Close all iterators that are outside the catch context.
            if let Some(active_iterator) = active_iterator {
                let inactive = context
                    .vm
                    .frame_mut()
                    .iterators
                    .split_off(active_iterator as usize + 1);
                for iterator in inactive {
                    if !iterator.done() {
                        drop(iterator.close(Ok(JsValue::undefined()), context));
                    }
                }
                context.vm.err.take();
            }

            let catch_target = context.vm.frame().env_stack[idx].start_address();

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
                    if current_address > env_entry.start_address() {
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
                context.vm.frame_mut().pc = catch_target;
            } else {
                context.vm.frame_mut().pc = target_address;
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
                if env_entry.start_address() < current_address {
                    target_address = Some(env_entry.exit_address());
                } else {
                    target_address = Some(env_entry.start_address());
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

            let active_iterator = context
                .vm
                .frame()
                .env_stack
                .iter()
                .filter_map(EnvStackEntry::iterator)
                .last();

            // Close all iterators that are outside the finally context.
            if let Some(active_iterator) = active_iterator {
                let inactive = context
                    .vm
                    .frame_mut()
                    .iterators
                    .split_off(active_iterator as usize + 1);
                for iterator in inactive {
                    if !iterator.done() {
                        drop(iterator.close(Ok(JsValue::undefined()), context));
                    }
                }
                context.vm.err.take();
            }

            let env_truncation_len = context.vm.environments.len().saturating_sub(env_to_pop);
            context.vm.environments.truncate(env_truncation_len);

            let previous_stack_size = context
                .vm
                .stack
                .len()
                .saturating_sub(context.vm.frame().pop_on_return as usize);
            context.vm.stack.truncate(previous_stack_size);
            context.vm.frame_mut().pop_on_return = 0;

            context.vm.frame_mut().pc = address;
            let err = error.to_opaque(context);
            context.vm.push(err);
            return Ok(CompletionType::Normal);
        }

        // Close all iterators that are still open.
        for iterator in std::mem::take(&mut context.vm.frame_mut().iterators) {
            if !iterator.done() {
                drop(iterator.close(Ok(JsValue::undefined()), context));
            }
        }
        context.vm.err.take();

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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
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
