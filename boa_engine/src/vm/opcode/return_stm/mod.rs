use crate::{
    vm::{opcode::Operation, FinallyReturn, ShouldExit},
    Context, JsResult,
};

/// `Return` implements the Opcode Operation for `Opcode::Return`
///
/// Operation:
///  - Return from a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Return;

impl Operation for Return {
    const NAME: &'static str = "Return";
    const INSTRUCTION: &'static str = "INST - Return";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let current_address = context.vm.frame().pc;
        let mut env_to_pop = 0;
        let mut finally_address = None;
        while !context.vm.frame().env_stack.is_empty() {
            let env_entry = context
                .vm
                .frame()
                .env_stack
                .last()
                .expect("EnvStackEntry must exist");

            if env_entry.is_finally_env() {
                if (env_entry.start_address() as usize) < current_address {
                    finally_address = Some(env_entry.exit_address() as usize);
                } else {
                    finally_address = Some(env_entry.start_address() as usize);
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

        if let Some(finally) = finally_address {
            context.vm.frame_mut().pc = finally;
            context.vm.frame_mut().finally_return = FinallyReturn::Ok;
            return Ok(ShouldExit::False);
        }

        Ok(ShouldExit::True)
    }
}
