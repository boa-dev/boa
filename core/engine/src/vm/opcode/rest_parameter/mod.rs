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
        let argument_count = context.vm.frame().argument_count;
        let param_count = context.vm.frame().code_block().parameter_length;

        let array = if argument_count >= param_count {
            let rest_count = argument_count - param_count + 1;
            let args = context.vm.pop_n_values(rest_count as usize);
            Array::create_array_from_list(args, context)
        } else {
            Array::array_create(0, None, context).expect("could not create an empty array")
        };

        context.vm.push(array);
        Ok(CompletionType::Normal)
    }
}
