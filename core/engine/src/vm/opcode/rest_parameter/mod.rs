use super::VaryingOperand;
use crate::{Context, builtins::Array, vm::opcode::Operation};

/// `RestParameterInit` implements the Opcode Operation for `Opcode::RestParameterInit`
///
/// Operation:
///  - Initialize the rest parameter value of a function from the remaining arguments.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RestParameterInit;

impl RestParameterInit {
    #[inline(always)]
    pub(super) fn operation(dst: VaryingOperand, context: &Context) {
        let array = if let Some(rest) = {
            let vm = context.vm_mut();
            vm.stack.pop_rest_arguments(&vm.frame)
        } {
            let rest_count = rest.len() as u32;
            let array = Array::create_array_from_list(rest, context);
            context.vm_mut().frame_mut().rp -= rest_count;
            context.vm_mut().frame_mut().argument_count -= rest_count;
            array
        } else {
            Array::array_create(0, None, context).expect("could not create an empty array")
        };
        context.vm_mut().set_register(dst.into(), array.into());
    }
}

impl Operation for RestParameterInit {
    const NAME: &'static str = "RestParameterInit";
    const INSTRUCTION: &'static str = "INST - RestParameterInit";
    const COST: u8 = 6;
}
