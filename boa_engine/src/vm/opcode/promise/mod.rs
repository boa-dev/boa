use crate::{
    vm::{opcode::Operation, FinallyReturn, ShouldExit},
    Context, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
            FinallyReturn::Ok => {
                Ok(ShouldExit::True)
            }
            FinallyReturn::Err => {
                Err(context.vm.pop())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
