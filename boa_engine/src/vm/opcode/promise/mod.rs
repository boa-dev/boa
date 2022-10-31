use crate::{
    vm::{opcode::Operation, FinallyReturn, ShouldExit},
    Context, JsError, JsResult,
};

/// `FinallyStart` implements the Opcode Operation for `Opcode::FinallyStart`
///
/// Operation:
///  - Start of a finally block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FinallyStart;

impl Operation for FinallyStart {
    const NAME: &'static str = "FinallyStart";
    const INSTRUCTION: &'static str = "INST - FinallyStart";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        *context
            .vm
            .frame_mut()
            .finally_jump
            .last_mut()
            .expect("finally jump must exist here") = None;
        Ok(ShouldExit::False)
    }
}

/// `FinallyEnd` implements the Opcode Operation for `Opcode::FinallyEnd`
///
/// Operation:
///  - End of a finally block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FinallyEnd;

impl Operation for FinallyEnd {
    const NAME: &'static str = "FinallyEnd";
    const INSTRUCTION: &'static str = "INST - FinallyEnd";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let address = context
            .vm
            .frame_mut()
            .finally_jump
            .pop()
            .expect("finally jump must exist here");
        match context.vm.frame_mut().finally_return {
            FinallyReturn::None => {
                if let Some(address) = address {
                    context.vm.frame_mut().pc = address as usize;
                }
                Ok(ShouldExit::False)
            }
            FinallyReturn::Ok => Ok(ShouldExit::True),
            FinallyReturn::Err => Err(JsError::from_opaque(context.vm.pop())),
        }
    }
}

/// `FinallySetJump` implements the Opcode Operation for `Opcode::FinallySetJump`
///
/// Operation:
///  - Set the address for a finally jump.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FinallySetJump;

impl Operation for FinallySetJump {
    const NAME: &'static str = "FinallySetJump";
    const INSTRUCTION: &'static str = "INST - FinallySetJump";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();
        *context
            .vm
            .frame_mut()
            .finally_jump
            .last_mut()
            .expect("finally jump must exist here") = Some(address);
        Ok(ShouldExit::False)
    }
}
