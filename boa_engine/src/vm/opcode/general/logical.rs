use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LogicalNot;

impl Operation for LogicalNot {
    const NAME: &'static str = "LogicalNot";
    const INSTRUCTION: &'static str = "INST - LogicalNot";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        context.vm.push(!value.to_boolean());
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LogicalAnd;

impl Operation for LogicalAnd {
    const NAME: &'static str = "LogicalAnd";
    const INSTRUCTION: &'static str = "INST - LogicalAnd";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.pop();
        if !lhs.to_boolean() {
            context.vm.frame_mut().pc = exit as usize;
            context.vm.push(lhs);
        }
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LogicalOr;

impl Operation for LogicalOr {
    const NAME: &'static str = "LogicalOr";
    const INSTRUCTION: &'static str = "INST - LogicalOr";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.pop();
        if lhs.to_boolean() {
            context.vm.frame_mut().pc = exit as usize;
            context.vm.push(lhs);
        }
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Coalesce;

impl Operation for Coalesce {
    const NAME: &'static str = "Coalesce";
    const INSTRUCTION: &'static str = "INST - Coalesce";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let exit = context.vm.read::<u32>();
        let lhs = context.vm.pop();
        if !lhs.is_null_or_undefined() {
            context.vm.frame_mut().pc = exit as usize;
            context.vm.push(lhs);
        }
        Ok(ShouldExit::False)
    }
}
