use crate::{
    builtins::Array,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `RestParameterInit` implements the Opcode Operation for `Opcode::RestParameterInit`
///
/// Operation:
///  - Initialize the rest parameter value of a function from the remaining arguments.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RestParameterInit;

impl Operation for RestParameterInit {
    const NAME: &'static str = "FunctionRestParameter";
    const INSTRUCTION: &'static str = "INST - FunctionRestParameter";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let arg_count = context.vm.frame().arg_count;
        let param_count = context.vm.frame().param_count;
        if arg_count >= param_count {
            let rest_count = arg_count - param_count + 1;
            let mut args = Vec::with_capacity(rest_count);
            for _ in 0..rest_count {
                args.push(context.vm.pop());
            }
            let array: _ = Array::create_array_from_list(args, context);

            context.vm.push(array);
        } else {
            context.vm.pop();

            let array =
                Array::array_create(0, None, context).expect("could not create an empty array");
            context.vm.push(array);
        }
        Ok(ShouldExit::False)
    }
}

/// `RestParameterPop` implements the Opcode Operation for `Opcode::RestParameterPop`
///
/// Operation:
///  - Pop the remaining arguments of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RestParameterPop;

impl Operation for RestParameterPop {
    const NAME: &'static str = "RestParameterPop";
    const INSTRUCTION: &'static str = "INST - RestParameterPop";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let arg_count = context.vm.frame().arg_count;
        let param_count = context.vm.frame().param_count;
        if arg_count > param_count {
            for _ in 0..(arg_count - param_count) {
                context.vm.pop();
            }
        }
        Ok(ShouldExit::False)
    }
}
