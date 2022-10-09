use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

pub(crate) mod if_thrown;

pub(crate) use if_thrown::PopIfThrown;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Pop;

impl Operation for Pop {
    const NAME: &'static str = "Pop";
    const INSTRUCTION: &'static str = "INST - Pop";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let _val = context.vm.pop();
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PopEnvironment;

impl Operation for PopEnvironment {
    const NAME: &'static str = "PopEnvironment";
    const INSTRUCTION: &'static str = "INST - PopEnvironment";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        context.realm.environments.pop();
        context.vm.frame_mut().loop_env_stack_dec();
        context.vm.frame_mut().try_env_stack_dec();
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PopOnReturnAdd;

impl Operation for PopOnReturnAdd {
    const NAME: &'static str = "PopOnReturnAdd";
    const INSTRUCTION: &'static str = "INST - PopOnReturnAdd";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        context.vm.frame_mut().pop_on_return += 1;
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PopOnReturnSub;

impl Operation for PopOnReturnSub {
    const NAME: &'static str = "PopOnReturnSub";
    const INSTRUCTION: &'static str = "INST - PopOnReturnSub";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        context.vm.frame_mut().pop_on_return -= 1;
        Ok(ShouldExit::False)
    }
}
