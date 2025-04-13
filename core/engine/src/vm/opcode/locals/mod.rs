use super::VaryingOperand;
use crate::{
    vm::{opcode::Operation, Registers},
    Context, JsNativeError, JsResult,
};

/// `PopIntoLocal` implements the Opcode Operation for `Opcode::PopIntoLocal`
///
/// Operation:
///  - Pop value from the stack and push to a local binding register `dst`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopIntoLocal;

impl PopIntoLocal {
    #[inline(always)]
    pub(super) fn operation(
        (src, dst): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) {
        context.vm.frame_mut().local_bindings_initialized[usize::from(dst)] = true;
        registers.set(dst.into(), registers.get(src.into()).clone());
    }
}

impl Operation for PopIntoLocal {
    const NAME: &'static str = "PopIntoLocal";
    const INSTRUCTION: &'static str = "INST - PopIntoLocal";
    const COST: u8 = 2;
}

/// `PushFromLocal` implements the Opcode Operation for `Opcode::PushFromLocal`
///
/// Operation:
///  - Copy value at local binding register `src` and push it into the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushFromLocal;

impl PushFromLocal {
    #[inline(always)]
    pub(super) fn operation(
        (src, dst): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        if !context.vm.frame().local_bindings_initialized[usize::from(src)] {
            return Err(JsNativeError::reference()
                .with_message("access to uninitialized binding")
                .into());
        }
        registers.set(dst.into(), registers.get(src.into()).clone());
        Ok(())
    }
}

impl Operation for PushFromLocal {
    const NAME: &'static str = "PushFromLocal";
    const INSTRUCTION: &'static str = "INST - PushFromLocal";
    const COST: u8 = 2;
}
