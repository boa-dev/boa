use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `Jump` implements the Opcode Operation for `Opcode::Jump`
///
/// Operation:
///  - Unconditional jump to address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Jump;

impl Operation for Jump {
    const NAME: &'static str = "Jump";
    const INSTRUCTION: &'static str = "INST - Jump";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        context.vm.frame_mut().pc = address as usize;
        Ok(ShouldExit::False)
    }
}

/// `JumpIfFalse` implements the Opcode Operation for `Opcode::JumpIfFalse`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct JumpIfFalse;

impl Operation for JumpIfFalse {
    const NAME: &'static str = "JumpIfFalse";
    const INSTRUCTION: &'static str = "INST - JumpIfFalse";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        if !context.vm.pop().to_boolean() {
            context.vm.frame_mut().pc = address as usize;
        }
        Ok(ShouldExit::False)
    }
}

/// `JumpIfNotUndefined` implements the Opcode Operation for `Opcode::JumpIfNotUndefined`
///
/// Operation:
///  - Conditional jump to address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct JumpIfNotUndefined;

impl Operation for JumpIfNotUndefined {
    const NAME: &'static str = "JumpIfNotUndefined";
    const INSTRUCTION: &'static str = "INST - JumpIfNotUndefined";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        let value = context.vm.pop();
        if !value.is_undefined() {
            context.vm.frame_mut().pc = address as usize;
            context.vm.push(value);
        }
        Ok(ShouldExit::False)
    }
}
