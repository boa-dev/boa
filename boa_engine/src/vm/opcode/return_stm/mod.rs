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
        if let Some(finally_address) = context.vm.frame().catch.last().and_then(|c| c.finally) {
            let frame = context.vm.frame_mut();
            frame.pc = finally_address as usize;
            frame.finally_return = FinallyReturn::Ok;
            frame.catch.pop();
            let mut envs_to_pop = 0_usize;
            for _ in 1..context.vm.frame().env_stack.len() {
                let env_entry = context.vm.frame_mut().env_stack.pop().expect("this must exist");
                envs_to_pop += env_entry.env_num();
    
                if env_entry.is_try_env() {
                    break;
                }
            }
    
            for _ in 0..envs_to_pop {
                context.realm.environments.pop();
            }
        } else {
            return Ok(ShouldExit::True);
        }
        Ok(ShouldExit::False)
    }
}
