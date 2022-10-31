use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `Case` implements the Opcode Operation for `Opcode::Case`
///
/// Operation:
///  - Pop the two values of the stack, strict equal compares the two values,
///    if true jumps to address, otherwise push the second pop'ed value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Case;

impl Operation for Case {
    const NAME: &'static str = "Case";
    const INSTRUCTION: &'static str = "INST - Case";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        let cond = context.vm.pop();
        let value = context.vm.pop();

        if value.strict_equals(&cond) {
            context.vm.frame_mut().pc = address as usize;
        } else {
            context.vm.push(value);
        }
        Ok(ShouldExit::False)
    }
}

/// `Default` implements the Opcode Operation for `Opcode::Default`
///
/// Operation:
///  - Pops the top of stack and jump to address.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Default;

impl Operation for Default {
    const NAME: &'static str = "Default";
    const INSTRUCTION: &'static str = "INST - Default";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let exit = context.vm.read::<u32>();
        let _val = context.vm.pop();
        context.vm.frame_mut().pc = exit as usize;
        Ok(ShouldExit::False)
    }
}
