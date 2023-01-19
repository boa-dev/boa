use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `Break` implements the Opcode Operation for `Opcode::Break`
///
/// Operation:
///   - Pop required environments and jump to address.
pub(crate) struct Break;

impl Operation for Break {
    const NAME: &'static str = "Break";
    const INSTRUCTION: &'static str = "INST - Break";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        let pop_envs = context.vm.read::<u32>();

        for _ in 0..pop_envs {
            context.realm.environments.pop();

            context.vm.frame_mut().dec_frame_env_stack();

            if context.vm.frame().env_stack.last().expect("must exist").env_num() == 0 {
                context.vm.frame_mut().env_stack.pop();
            }
        }

        context.vm.frame_mut().pc = address as usize;
        Ok(ShouldExit::False)
    }
}
