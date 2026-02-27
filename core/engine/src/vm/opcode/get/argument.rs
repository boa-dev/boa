use crate::{
    Context,
    vm::opcode::{Operation, VaryingOperand},
};

/// `GetArgument` implements the Opcode Operation for `Opcode::GetArgument`
///
/// Operation:
///  - Get i-th argument of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetArgument;

impl GetArgument {
    #[inline(always)]
    pub(crate) fn operation((index, dst): (VaryingOperand, VaryingOperand), context: &Context) {
        let vm = context.vm_mut();
        let value = vm
            .stack
            .get_argument(&vm.frame, index.into())
            .cloned()
            .unwrap_or_default();
        vm.set_register(dst.into(), value);
    }
}

impl Operation for GetArgument {
    const NAME: &'static str = "GetArgument";
    const INSTRUCTION: &'static str = "INST - GetArgument";
    const COST: u8 = 2;
}
