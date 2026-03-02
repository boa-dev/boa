use super::VaryingOperand;
use crate::{Context, vm::opcode::Operation};

/// `Case` implements the Opcode Operation for `Opcode::Case`
///
/// Operation:
///  - Pop the two values of the stack, strict equal compares the two values,
///    if true jumps to address, otherwise push the second pop'ed value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Case;

impl Case {
    #[inline(always)]
    pub(super) fn operation(
        (address, value, condition): (u32, VaryingOperand, VaryingOperand),
        context: &Context,
    ) {
        let equals = {
            let vm = unsafe { &*context.vm_ptr() };
            vm.get_register(value.into())
                .strict_equals(vm.get_register(condition.into()))
        };
        if equals {
            context.set_pc(address);
        }
    }
}

impl Operation for Case {
    const NAME: &'static str = "Case";
    const INSTRUCTION: &'static str = "INST - Case";
    const COST: u8 = 2;
}
