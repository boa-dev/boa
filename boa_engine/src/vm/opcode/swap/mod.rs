use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `Swap` implements the Opcode Operation for `Opcode::Swap`
///
/// Operation:
///  - Swap the top two values on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Swap;

impl Operation for Swap {
    const NAME: &'static str = "Swap";
    const INSTRUCTION: &'static str = "INST - Swap";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let len = context.vm.stack.len();
        assert!(len > 1);
        context.vm.stack.swap(len - 1, len - 2);
        Ok(CompletionType::Normal)
    }
}

/// `RotateLeft` implements the Opcode Operation for `Opcode::RotateLeft`
///
/// Operation:
///  - Rotates the n top values to the left.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RotateLeft;

impl Operation for RotateLeft {
    const NAME: &'static str = "RotateLeft";
    const INSTRUCTION: &'static str = "INST - RotateLeft";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let n = context.vm.read::<u8>() as usize;
        let len = context.vm.stack.len();
        context.vm.stack[(len - n)..].rotate_left(1);
        Ok(CompletionType::Normal)
    }
}

/// `RotateRight` implements the Opcode Operation for `Opcode::RotateRight`
///
/// Operation:
///  - Rotates the n top values to the right.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RotateRight;

impl Operation for RotateRight {
    const NAME: &'static str = "RotateRight";
    const INSTRUCTION: &'static str = "INST - RotateRight";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let n = context.vm.read::<u8>() as usize;
        let len = context.vm.stack.len();
        context.vm.stack[(len - n)..].rotate_right(1);
        Ok(CompletionType::Normal)
    }
}
