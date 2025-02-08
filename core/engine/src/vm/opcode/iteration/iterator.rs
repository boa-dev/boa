use crate::{
    builtins::{iterable::create_iter_result_object, Array},
    js_string,
    vm::{opcode::Operation, CompletionType, GeneratorResumeKind, Registers},
    Context, JsResult,
};

/// `IteratorNext` implements the Opcode Operation for `Opcode::IteratorNext`
///
/// Operation:
///  - Calls the `next` method of `iterator`, updating its record with the next value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorNext;

impl Operation for IteratorNext {
    const NAME: &'static str = "IteratorNext";
    const INSTRUCTION: &'static str = "INST - IteratorNext";
    const COST: u8 = 6;

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");

        iterator.step(context)?;

        context.vm.frame_mut().iterators.push(iterator);

        Ok(CompletionType::Normal)
    }
}

/// `IteratorFinishAsyncNext` implements the Opcode Operation for `Opcode::IteratorFinishAsyncNext`.
///
/// Operation:
///  - Finishes the call to `Opcode::IteratorNext` within a `for await` loop by setting the current
///    result of the current iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorFinishAsyncNext;

impl IteratorFinishAsyncNext {
    fn operation(
        resume_kind: u32,
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        let resume_kind = registers.get(resume_kind).to_generator_resume_kind();

        if matches!(resume_kind, GeneratorResumeKind::Throw) {
            // If after awaiting the `next` call the iterator returned an error, it can be considered
            // as poisoned, meaning we can remove it from the iterator stack to avoid calling
            // cleanup operations on it.
            return Ok(CompletionType::Normal);
        }

        let value = registers.get(value);
        iterator.update_result(value.clone(), context)?;
        context.vm.frame_mut().iterators.push(iterator);
        Ok(CompletionType::Normal)
    }
}

impl Operation for IteratorFinishAsyncNext {
    const NAME: &'static str = "IteratorFinishAsyncNext";
    const INSTRUCTION: &'static str = "INST - IteratorFinishAsyncNext";
    const COST: u8 = 5;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let resume_kind = context.vm.read::<u8>().into();
        let value = context.vm.read::<u8>().into();
        Self::operation(resume_kind, value, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let resume_kind = context.vm.read::<u16>().into();
        let value = context.vm.read::<u16>().into();
        Self::operation(resume_kind, value, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let resume_kind = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        Self::operation(resume_kind, value, registers, context)
    }
}

/// `IteratorResult` implements the Opcode Operation for `Opcode::IteratorResult`
///
/// Operation:
///  - Gets the last iteration result of the current iterator record.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorResult;

impl IteratorResult {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let last_result = context
            .vm
            .frame()
            .iterators
            .last()
            .expect("iterator on the call frame must exist")
            .last_result()
            .object()
            .clone();
        registers.set(value, last_result.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for IteratorResult {
    const NAME: &'static str = "IteratorResult";
    const INSTRUCTION: &'static str = "INST - IteratorResult";
    const COST: u8 = 3;

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

/// `IteratorValue` implements the Opcode Operation for `Opcode::IteratorValue`
///
/// Operation:
///  - Gets the `value` property of the current iterator record.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorValue;

impl IteratorValue {
    fn operation(
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        let iter_value = iterator.value(context)?;
        registers.set(value, iter_value);

        context.vm.frame_mut().iterators.push(iterator);

        Ok(CompletionType::Normal)
    }
}

impl Operation for IteratorValue {
    const NAME: &'static str = "IteratorValue";
    const INSTRUCTION: &'static str = "INST - IteratorValue";
    const COST: u8 = 5;

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

/// `IteratorDone` implements the Opcode Operation for `Opcode::IteratorDone`
///
/// Operation:
///  - Returns `true` if the current iterator is done, or `false` otherwise
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorDone;

impl IteratorDone {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        done: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = context
            .vm
            .frame()
            .iterators
            .last()
            .expect("iterator on the call frame must exist")
            .done();
        registers.set(done, value.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for IteratorDone {
    const NAME: &'static str = "IteratorDone";
    const INSTRUCTION: &'static str = "INST - IteratorDone";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let done = context.vm.read::<u8>().into();
        Self::operation(done, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let done = context.vm.read::<u16>().into();
        Self::operation(done, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let done = context.vm.read::<u32>();
        Self::operation(done, registers, context)
    }
}

/// `IteratorReturn` implements the Opcode Operation for `Opcode::IteratorReturn`
///
/// Operation:
///  - Calls `return` on the current iterator and returns the result.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorReturn;

impl IteratorReturn {
    fn operation(
        value: u32,
        called: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let Some(record) = context.vm.frame_mut().iterators.pop() else {
            registers.set(called, false.into());
            return Ok(CompletionType::Normal);
        };

        if record.done() {
            registers.set(called, false.into());
            return Ok(CompletionType::Normal);
        }

        let Some(ret) = record
            .iterator()
            .get_method(js_string!("return"), context)?
        else {
            registers.set(called, false.into());
            return Ok(CompletionType::Normal);
        };

        let old_return_value = context.vm.get_return_value();

        let return_value = ret.call(&record.iterator().clone().into(), &[], context)?;

        context.vm.set_return_value(old_return_value);

        registers.set(value, return_value);
        registers.set(called, true.into());

        Ok(CompletionType::Normal)
    }
}

impl Operation for IteratorReturn {
    const NAME: &'static str = "IteratorReturn";
    const INSTRUCTION: &'static str = "INST - IteratorReturn";
    const COST: u8 = 8;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let called = context.vm.read::<u8>().into();
        Self::operation(value, called, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let called = context.vm.read::<u16>().into();
        Self::operation(value, called, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let called = context.vm.read::<u32>();
        Self::operation(value, called, registers, context)
    }
}

/// `IteratorToArray` implements the Opcode Operation for `Opcode::IteratorToArray`
///
/// Operation:
///  - Consume the iterator and construct and array with all the values.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorToArray;

impl IteratorToArray {
    fn operation(
        array: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        let mut values = Vec::new();

        loop {
            let done = match iterator.step(context) {
                Ok(done) => done,
                Err(err) => {
                    context.vm.frame_mut().iterators.push(iterator);
                    return Err(err);
                }
            };

            if done {
                break;
            }

            match iterator.value(context) {
                Ok(value) => values.push(value),
                Err(err) => {
                    context.vm.frame_mut().iterators.push(iterator);
                    return Err(err);
                }
            }
        }

        context.vm.frame_mut().iterators.push(iterator);
        let result = Array::create_array_from_list(values, context);
        registers.set(array, result.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for IteratorToArray {
    const NAME: &'static str = "IteratorToArray";
    const INSTRUCTION: &'static str = "INST - IteratorToArray";
    const COST: u8 = 8;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let array = context.vm.read::<u8>().into();
        Self::operation(array, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let array = context.vm.read::<u16>().into();
        Self::operation(array, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let array = context.vm.read::<u32>();
        Self::operation(array, registers, context)
    }
}

/// `IteratorStackEmpty` implements the Opcode Operation for `Opcode::IteratorStackEmpty`
///
/// Operation:
/// - Pushes `true` to the stack if the iterator stack is empty.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorStackEmpty;

impl IteratorStackEmpty {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        empty: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let is_empty = context.vm.frame().iterators.is_empty();
        registers.set(empty, is_empty.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for IteratorStackEmpty {
    const NAME: &'static str = "IteratorStackEmpty";
    const INSTRUCTION: &'static str = "INST - IteratorStackEmpty";
    const COST: u8 = 1;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let empty = context.vm.read::<u8>().into();
        Self::operation(empty, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let empty = context.vm.read::<u16>().into();
        Self::operation(empty, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let empty = context.vm.read::<u32>();
        Self::operation(empty, registers, context)
    }
}

/// `CreateIteratorResult` implements the Opcode Operation for `Opcode::CreateIteratorResult`
///
/// Operation:
/// -  Creates a new iterator result object
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateIteratorResult;

impl CreateIteratorResult {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        value: u32,
        done: bool,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let val = registers.get(value);
        let result = create_iter_result_object(val.clone(), done, context);
        registers.set(value, result);
        Ok(CompletionType::Normal)
    }
}

impl Operation for CreateIteratorResult {
    const NAME: &'static str = "CreateIteratorResult";
    const INSTRUCTION: &'static str = "INST - CreateIteratorResult";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let done = context.vm.read::<u8>() != 0;
        Self::operation(value, done, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let done = context.vm.read::<u8>() != 0;
        Self::operation(value, done, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let done = context.vm.read::<u8>() != 0;
        Self::operation(value, done, registers, context)
    }
}
