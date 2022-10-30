use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `Swap` implements the Opcode Operation for `Opcode::Swap`
///
/// Operation:
///  - Swap the top two values on the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Swap;

impl Operation for Swap {
    const NAME: &'static str = "Swap";
    const INSTRUCTION: &'static str = "INST - Swap";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let first = context.vm.pop();
        let second = context.vm.pop();

        context.vm.push(first);
        context.vm.push(second);
        Ok(ShouldExit::False)
    }
}

/// `Swap3` implements the Opcode Operation for `Opcode::Swap3`
///
/// Operation:
///  - Swap the top three values on the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Swap3;

impl Operation for Swap3 {
    const NAME: &'static str = "Swap3";
    const INSTRUCTION: &'static str = "INST - Swap3";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let len = context.vm.stack.len();
        context.vm.stack.swap(len - 1, len - 3);
        Ok(ShouldExit::False)
    }
}

/// `RotateLeft` implements the Opcode Operation for `Opcode::RotateLeft`
///
/// Operation:
///  - Rotates the n top values to the left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct RotateLeft;

impl Operation for RotateLeft {
    const NAME: &'static str = "RotateLeft";
    const INSTRUCTION: &'static str = "INST - RotateLeft";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let n = context.vm.read::<u8>() as usize;
        let len = context.vm.stack.len();
        context.vm.stack[(len - n)..].rotate_left(1);
        Ok(ShouldExit::False)
    }
}

/// `RotateRight` implements the Opcode Operation for `Opcode::RotateRight`
///
/// Operation:
///  - Rotates the n top values to the right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct RotateRight;

impl Operation for RotateRight {
    const NAME: &'static str = "RotateRight";
    const INSTRUCTION: &'static str = "INST - RotateRight";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let n = context.vm.read::<u8>() as usize;
        let len = context.vm.stack.len();
        context.vm.stack[(len - n)..].rotate_right(1);
        Ok(ShouldExit::False)
    }
}
