use crate::{
    builtins::OrdinaryObject,
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionType, Registers,
    },
    Context, JsResult,
};

/// `PushEmptyObject` implements the Opcode Operation for `Opcode::PushEmptyObject`
///
/// Operation:
///  - Push empty object `{}` value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushEmptyObject;

impl PushEmptyObject {
    #[allow(clippy::unnecessary_wraps)]
    #[inline(always)]
    pub(crate) fn operation(
        dst: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let o = context
            .intrinsics()
            .templates()
            .ordinary_object()
            .create(OrdinaryObject, Vec::default());
        registers.set(dst.into(), o.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushEmptyObject {
    const NAME: &'static str = "PushEmptyObject";
    const INSTRUCTION: &'static str = "INST - PushEmptyObject";
    const COST: u8 = 1;
}
