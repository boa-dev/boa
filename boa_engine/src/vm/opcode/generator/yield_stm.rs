use crate::{
    builtins::async_generator::{AsyncGenerator, AsyncGeneratorState},
    vm::{opcode::Operation, CompletionRecord, CompletionType, GeneratorResumeKind},
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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.pop();

        let async_gen = context
            .vm
            .frame()
            .async_generator
            .clone()
            .expect("`AsyncGeneratorYield` must only be called inside async generators");
        let completion = Ok(value);
        let next = async_gen
            .borrow_mut()
            .as_async_generator_mut()
            .expect("must be async generator object")
            .queue
            .pop_front()
            .expect("must have item in queue");

        // TODO: 7. Let previousContext be the second to top element of the execution context stack.
        AsyncGenerator::complete_step(&next, completion, false, None, context);

        let mut generator_object_mut = async_gen.borrow_mut();
        let gen = generator_object_mut
            .as_async_generator_mut()
            .expect("must be async generator object");

        if let Some(next) = gen.queue.front() {
            let resume_kind = match next.completion.clone() {
                CompletionRecord::Normal(val) => {
                    context.vm.push(val);
                    GeneratorResumeKind::Normal
                }
                CompletionRecord::Return(val) => {
                    context.vm.push(val);
                    GeneratorResumeKind::Return
                }
                CompletionRecord::Throw(err) => {
                    let err = err.to_opaque(context);
                    context.vm.push(err);
                    GeneratorResumeKind::Throw
                }
            };

            context.vm.push(resume_kind);

            return Ok(CompletionType::Normal);
        }

        gen.state = AsyncGeneratorState::SuspendedYield;
        context.vm.set_return_value(JsValue::undefined());
        Ok(CompletionType::Yield)
    }
}
