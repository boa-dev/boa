use crate::{
    builtins::async_generator::{AsyncGenerator, AsyncGeneratorState},
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

/// `GeneratorYield` implements the Opcode Operation for `Opcode::GeneratorYield`
///
/// Operation:
///  - Yield from the current generator execution.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorYield;

impl Operation for GeneratorYield {
    const NAME: &'static str = "GeneratorYield";
    const INSTRUCTION: &'static str = "INST - GeneratorYield";
    const COST: u8 = 1;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        context.vm.set_return_value(value);
        Ok(CompletionType::Yield)
    }
}

/// `AsyncGeneratorYield` implements the Opcode Operation for `Opcode::AsyncGeneratorYield`
///
/// Operation:
///  - Yield from the current async generator execution.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AsyncGeneratorYield;

impl Operation for AsyncGeneratorYield {
    const NAME: &'static str = "AsyncGeneratorYield";
    const INSTRUCTION: &'static str = "INST - AsyncGeneratorYield";
    const COST: u8 = 8;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();

        let async_generator_object = context
            .vm
            .frame()
            .async_generator_object(&context.vm.stack)
            .expect("`AsyncGeneratorYield` must only be called inside async generators");
        let completion = Ok(value);
        let async_generator_object = async_generator_object
            .downcast::<AsyncGenerator>()
            .expect("must be async generator object");
        let next = async_generator_object
            .borrow_mut()
            .data
            .queue
            .pop_front()
            .expect("must have item in queue");

        async_generator_object.borrow_mut().data.state = AsyncGeneratorState::SuspendedYield;

        // TODO: 7. Let previousContext be the second to top element of the execution context stack.
        AsyncGenerator::complete_step(&next, completion, false, None, context);

        // TODO: Upgrade to the latest spec when the problem is fixed.
        AsyncGenerator::resume_next(&async_generator_object, context);

        context.vm.set_return_value(JsValue::undefined());
        Ok(CompletionType::Yield)
    }
}
