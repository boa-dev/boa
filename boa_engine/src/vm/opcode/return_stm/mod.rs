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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        if let Some(finally_address) = context.vm.frame().catch.last().and_then(|c| c.finally) {
            let frame = context.vm.frame_mut();
            frame.pc = finally_address as usize;
            frame.finally_return = FinallyReturn::Ok;
            frame.catch.pop();
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
        } else {
            return Ok(ShouldExit::True);
        }
        Ok(ShouldExit::False)
    }
}
