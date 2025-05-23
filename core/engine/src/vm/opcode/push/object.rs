use crate::{
    builtins::OrdinaryObject,
    vm::{
        opcode::{Operation, VaryingOperand},
        Registers,
    },
    Context,
};

/// `PushEmptyObject` implements the Opcode Operation for `Opcode::PushEmptyObject`
///
/// Operation:
///  - Push empty object `{}` value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushEmptyObject;

impl PushEmptyObject {
    #[inline(always)]
    pub(crate) fn operation(dst: VaryingOperand, context: &mut Context) {
        let o = context
            .intrinsics()
            .templates()
            .ordinary_object()
            .create(OrdinaryObject, Vec::default());
        let fp = context.vm.frame().fp() as usize;
        context.vm.stack[fp + dst.value as usize] = o.into();
    }
}

impl Operation for PushEmptyObject {
    const NAME: &'static str = "PushEmptyObject";
    const INSTRUCTION: &'static str = "INST - PushEmptyObject";
    const COST: u8 = 1;
}
