use crate::{
    vm::{call_frame::EnvStackEntry, opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `LabelledStart` implements the Opcode Operation for `Opcode::LabelledStart`
///
/// Operation:
///  - Start of a labelled block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LabelledStart;

impl Operation for LabelledStart {
    const NAME: &'static str = "LabelledStart";
    const INSTRUCTION: &'static str = "INST - LabelledStart";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let start = context.vm.frame().pc as u32 - 1;
        let finally = context.vm.read::<u32>();
        context
            .vm
            .frame_mut()
            .env_stack
            .push(EnvStackEntry::new(start, finally).with_labelled_flag());
        Ok(ShouldExit::False)
    }
}

/// `LabelledEnd` implements the Opcode Operation for `Opcode::LabelledEnd`
///
/// Operation:
///  - Clean up enviroments at the end of labelled block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LabelledEnd;

impl Operation for LabelledEnd {
    const NAME: &'static str = "LabelledEnd";
    const INSTRUCTION: &'static str = "INST - LabelledEnd";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let mut envs_to_pop = 0_usize;
        for _ in 1..context.vm.frame().env_stack.len() {
            let env_entry = context
                .vm
                .frame_mut()
                .env_stack
                .pop()
                .expect("this must exist");
            envs_to_pop += env_entry.env_num();

            if env_entry.is_labelled_env() {
                break;
            }
        }

        for _ in 0..envs_to_pop {
            context.realm.environments.pop();
        }
        Ok(ShouldExit::False)
    }
}
