use crate::{
    builtins::Array,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `RestParameterInit` implements the Opcode Operation for `Opcode::RestParameterInit`
///
/// Operation:
///  - Initialize the rest parameter value of a function from the remaining arguments.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RestParameterInit;

impl Operation for RestParameterInit {
    const NAME: &'static str = "RestParameterInit";
    const INSTRUCTION: &'static str = "INST - RestParameterInit";
    const COST: u8 = 6;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let argument_count = context.vm.frame().argument_count as usize;
        let register_count = context.vm.frame().code_block().register_count as usize;
        let param_count = context.vm.frame().code_block().params.as_ref().len();

        if argument_count >= param_count {
            let rest_count = argument_count - param_count + 1;

            let len = context.vm.stack.len();
            let start = len - rest_count - register_count;
            let end = len - register_count;

            let args = &context.vm.stack[start..end];

            let array = Array::create_array_from_list(args.iter().cloned(), context);
            context.vm.stack.splice(start..end, [array.clone().into()]);

            context.vm.frame_mut().rp -= (start..end).len() as u32;
            context.vm.frame_mut().argument_count -= (start..end).len() as u32;

            context.vm.push(array);
        } else {
            let array =
                Array::array_create(0, None, context).expect("could not create an empty array");
            context.vm.push(array);
        }

        Ok(CompletionType::Normal)
    }
}
