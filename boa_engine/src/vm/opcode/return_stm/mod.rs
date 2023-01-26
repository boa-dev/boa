use crate::{
    vm::{call_frame::TryAddresses, opcode::Operation, FinallyReturn, ShouldExit},
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
        if let Some(finally_address) = context
            .vm
            .frame()
            .try_catch
            .last()
            .and_then(TryAddresses::finally)
        {
            let frame = context.vm.frame_mut();
            frame.pc = finally_address as usize;
            frame.finally_return = FinallyReturn::Ok;
            frame.try_catch.pop();

            let env_entry = context
                .vm
                .frame_mut()
                .env_stack
                .pop()
                .expect("this must exist");

            for _ in 0..env_entry.env_num() {
                context.realm.environments.pop();
            }
        } else {
            return Ok(ShouldExit::True);
        }
        Ok(ShouldExit::False)
    }
}
