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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        context.vm.frame_mut().r#yield = true;
        Ok(CompletionType::Return)
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let value = raw_context.vm.pop();

        let async_gen = raw_context
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

        let context = context.as_raw_context_mut();

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

            context.vm.frame_mut().generator_resume_kind = resume_kind;

            Ok(CompletionType::Normal)
        } else {
            gen.state = AsyncGeneratorState::SuspendedYield;
            context.vm.push(JsValue::undefined());
            GeneratorYield::execute(context)
        }
    }
}
