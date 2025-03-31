use super::VaryingOperand;
use crate::{
    builtins::Array,
    vm::{opcode::Operation, Registers},
    Context,
};

/// `RestParameterInit` implements the Opcode Operation for `Opcode::RestParameterInit`
///
/// Operation:
///  - Initialize the rest parameter value of a function from the remaining arguments.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RestParameterInit;

impl RestParameterInit {
    #[inline(always)]
    pub(super) fn operation(dst: VaryingOperand, registers: &mut Registers, context: &mut Context) {
        let frame = context.vm.frame();
        let argument_count = frame.argument_count;
        let param_count = frame.code_block().parameter_length;
        let array = if argument_count >= param_count {
            let rest_count = argument_count - param_count + 1;

            let len = context.vm.stack.len() as u32;
            let start = (len - rest_count) as usize;
            let end = len as usize;

            let args = &context.vm.stack[start..end];

            let array = Array::create_array_from_list(args.iter().cloned(), context);
            context.vm.stack.drain(start..end);

            context.vm.frame_mut().rp -= (start..end).len() as u32;
            context.vm.frame_mut().argument_count -= (start..end).len() as u32;

            array
        } else {
            Array::array_create(0, None, context).expect("could not create an empty array")
        };

        registers.set(dst.into(), array.into());
    }
}

impl Operation for RestParameterInit {
    const NAME: &'static str = "RestParameterInit";
    const INSTRUCTION: &'static str = "INST - RestParameterInit";
    const COST: u8 = 6;
}
