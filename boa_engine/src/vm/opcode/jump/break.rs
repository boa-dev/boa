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
        let envs = context.vm.read::<u32>();

        for _i in 0..envs {
            context.realm.environments.pop();
            context.vm.frame_mut().loop_env_stack_dec();
            context.vm.frame_mut().try_env_stack_dec();
        }
        context.vm.frame_mut().pc = address as usize;
        Ok(ShouldExit::False)
    }
}
