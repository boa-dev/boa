use crate::{
    Context,
    vm::opcode::{Operation, VaryingOperand},
};

/// `LogicalNot` implements the Opcode Operation for `Opcode::LogicalNot`
///
/// Operation:
///  - Unary logical `!` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogicalNot;

impl LogicalNot {
    #[inline(always)]
    pub(crate) fn operation(value: VaryingOperand, context: &Context) {
        // SAFETY: No other references to the VM exist during this block.
        unsafe {
            let vm = &mut *context.vm_ptr();
            let result = (!vm.get_register(value.into()).to_boolean()).into();
            vm.set_register(value.into(), result);
        }
    }
}

impl Operation for LogicalNot {
    const NAME: &'static str = "LogicalNot";
    const INSTRUCTION: &'static str = "INST - LogicalNot";
    const COST: u8 = 1;
}
