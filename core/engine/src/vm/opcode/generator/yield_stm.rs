use crate::{
    builtins::async_generator::{AsyncGenerator, AsyncGeneratorState},
    vm::{opcode::Operation, CompletionRecord, CompletionType, GeneratorResumeKind, Registers},
    Context, JsResult, JsValue,
};

/// `GeneratorYield` implements the Opcode Operation for `Opcode::GeneratorYield`
///
/// Operation:
///  - Yield from the current generator execution.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeneratorYield;

impl GeneratorYield {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        context.vm.set_return_value(value.clone());
        Ok(CompletionType::Yield)
    }
}

impl Operation for GeneratorYield {
    const NAME: &'static str = "GeneratorYield";
    const INSTRUCTION: &'static str = "INST - GeneratorYield";
    const COST: u8 = 1;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        Self::operation(value, registers, context)
    }
}

/// `AsyncGeneratorYield` implements the Opcode Operation for `Opcode::AsyncGeneratorYield`
///
/// Operation:
///  - Yield from the current async generator execution.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AsyncGeneratorYield;

impl AsyncGeneratorYield {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        // AsyncGeneratorYield ( value )
        // https://tc39.es/ecma262/#sec-asyncgeneratoryield

        // 1. Let genContext be the running execution context.
        // 2. Assert: genContext is the execution context of a generator.
        // 3. Let generator be the value of the Generator component of genContext.
        // 4. Assert: GetGeneratorKind() is async.
        let async_generator_object = context
            .vm
            .frame()
            .async_generator_object(registers)
            .expect("`AsyncGeneratorYield` must only be called inside async generators");
        let async_generator_object = async_generator_object
            .downcast::<AsyncGenerator>()
            .expect("must be async generator object");

        // 5. Let completion be NormalCompletion(value).
        let value = registers.get(value);
        let completion = Ok(value.clone());

        // TODO: 6. Assert: The execution context stack has at least two elements.
        // TODO: 7. Let previousContext be the second to top element of the execution context stack.
        // TODO: 8. Let previousRealm be previousContext's Realm.
        // 9. Perform AsyncGeneratorCompleteStep(generator, completion, false, previousRealm).
        AsyncGenerator::complete_step(&async_generator_object, completion, false, None, context);

        let mut gen = async_generator_object.borrow_mut();

        // 10. Let queue be generator.[[AsyncGeneratorQueue]].
        // 11. If queue is not empty, then
        //     a. NOTE: Execution continues without suspending the generator.
        //     b. Let toYield be the first element of queue.
        if let Some(next) = gen.data.queue.front() {
            // c. Let resumptionValue be Completion(toYield.[[Completion]]).
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

            // d. Return ? AsyncGeneratorUnwrapYieldResumption(resumptionValue).
            return Ok(CompletionType::Normal);
        }

        // 12. Else,

        //     a. Set generator.[[AsyncGeneratorState]] to suspended-yield.
        gen.data.state = AsyncGeneratorState::SuspendedYield;

        //     TODO: b. Remove genContext from the execution context stack and restore the execution context that is at the top of the execution context stack as the running execution context.
        //     TODO: c. Let callerContext be the running execution context.
        //     d. Resume callerContext passing undefined. If genContext is ever resumed again, let resumptionValue be the Completion Record with which it is resumed.
        //     e. Assert: If control reaches here, then genContext is the running execution context again.
        //     f. Return ? AsyncGeneratorUnwrapYieldResumption(resumptionValue).
        context.vm.set_return_value(JsValue::undefined());
        Ok(CompletionType::Yield)
    }
}

impl Operation for AsyncGeneratorYield {
    const NAME: &'static str = "AsyncGeneratorYield";
    const INSTRUCTION: &'static str = "INST - AsyncGeneratorYield";
    const COST: u8 = 8;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        Self::operation(value, registers, context)
    }
}
