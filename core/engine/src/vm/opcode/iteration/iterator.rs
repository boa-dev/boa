use crate::{
    builtins::{iterable::create_iter_result_object, Array},
    js_string,
    vm::{
        opcode::{Operation, VaryingOperand},
        GeneratorResumeKind, Registers,
    },
    Context, JsResult,
};

/// `IteratorNext` implements the Opcode Operation for `Opcode::IteratorNext`
///
/// Operation:
///  - Calls the `next` method of `iterator`, updating its record with the next value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorNext;

impl IteratorNext {
    #[inline(always)]
    pub(crate) fn operation((): (), _: &mut Registers, context: &mut Context) -> JsResult<()> {
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");

        iterator.step(context)?;

        context.vm.frame_mut().iterators.push(iterator);

        Ok(())
    }
}

impl Operation for IteratorNext {
    const NAME: &'static str = "IteratorNext";
    const INSTRUCTION: &'static str = "INST - IteratorNext";
    const COST: u8 = 6;
}

/// `IteratorFinishAsyncNext` implements the Opcode Operation for `Opcode::IteratorFinishAsyncNext`.
///
/// Operation:
///  - Finishes the call to `Opcode::IteratorNext` within a `for await` loop by setting the current
///    result of the current iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorFinishAsyncNext;

impl IteratorFinishAsyncNext {
    #[inline(always)]
    pub(crate) fn operation(
        (resume_kind, value): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        let resume_kind = registers.get(resume_kind.into()).to_generator_resume_kind();

        if matches!(resume_kind, GeneratorResumeKind::Throw) {
            // If after awaiting the `next` call the iterator returned an error, it can be considered
            // as poisoned, meaning we can remove it from the iterator stack to avoid calling
            // cleanup operations on it.
            return Ok(());
        }

        let value = registers.get(value.into());
        iterator.update_result(value.clone(), context)?;
        context.vm.frame_mut().iterators.push(iterator);
        Ok(())
    }
}

impl Operation for IteratorFinishAsyncNext {
    const NAME: &'static str = "IteratorFinishAsyncNext";
    const INSTRUCTION: &'static str = "INST - IteratorFinishAsyncNext";
    const COST: u8 = 5;
}

/// `IteratorResult` implements the Opcode Operation for `Opcode::IteratorResult`
///
/// Operation:
///  - Gets the last iteration result of the current iterator record.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorResult;

impl IteratorResult {
    #[inline(always)]
    pub(crate) fn operation(
        value: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let last_result = context
            .vm
            .frame()
            .iterators
            .last()
            .expect("iterator on the call frame must exist")
            .last_result()
            .object()
            .clone();
        registers.set(value.into(), last_result.into());
    }
}

impl Operation for IteratorResult {
    const NAME: &'static str = "IteratorResult";
    const INSTRUCTION: &'static str = "INST - IteratorResult";
    const COST: u8 = 3;
}

/// `IteratorValue` implements the Opcode Operation for `Opcode::IteratorValue`
///
/// Operation:
///  - Gets the `value` property of the current iterator record.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorValue;

impl IteratorValue {
    #[inline(always)]
    pub(crate) fn operation(
        value: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        let iter_value = iterator.value(context)?;
        registers.set(value.into(), iter_value);

        context.vm.frame_mut().iterators.push(iterator);

        Ok(())
    }
}

impl Operation for IteratorValue {
    const NAME: &'static str = "IteratorValue";
    const INSTRUCTION: &'static str = "INST - IteratorValue";
    const COST: u8 = 5;
}

/// `IteratorDone` implements the Opcode Operation for `Opcode::IteratorDone`
///
/// Operation:
///  - Returns `true` if the current iterator is done, or `false` otherwise
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorDone;

impl IteratorDone {
    #[inline(always)]
    pub(crate) fn operation(
        done: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let value = context
            .vm
            .frame()
            .iterators
            .last()
            .expect("iterator on the call frame must exist")
            .done();
        registers.set(done.into(), value.into());
    }
}

impl Operation for IteratorDone {
    const NAME: &'static str = "IteratorDone";
    const INSTRUCTION: &'static str = "INST - IteratorDone";
    const COST: u8 = 3;
}

/// `IteratorReturn` implements the Opcode Operation for `Opcode::IteratorReturn`
///
/// Operation:
///  - Calls `return` on the current iterator and returns the result.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorReturn;

impl IteratorReturn {
    #[inline(always)]
    pub(crate) fn operation(
        (value, called): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let Some(record) = context.vm.frame_mut().iterators.pop() else {
            registers.set(called.into(), false.into());
            return Ok(());
        };

        if record.done() {
            registers.set(called.into(), false.into());
            return Ok(());
        }

        let Some(ret) = record
            .iterator()
            .get_method(js_string!("return"), context)?
        else {
            registers.set(called.into(), false.into());
            return Ok(());
        };

        let old_return_value = context.vm.get_return_value();

        let return_value = ret.call(&record.iterator().clone().into(), &[], context)?;

        context.vm.set_return_value(old_return_value);

        registers.set(value.into(), return_value);
        registers.set(called.into(), true.into());

        Ok(())
    }
}

impl Operation for IteratorReturn {
    const NAME: &'static str = "IteratorReturn";
    const INSTRUCTION: &'static str = "INST - IteratorReturn";
    const COST: u8 = 8;
}

/// `IteratorToArray` implements the Opcode Operation for `Opcode::IteratorToArray`
///
/// Operation:
///  - Consume the iterator and construct and array with all the values.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorToArray;

impl IteratorToArray {
    #[inline(always)]
    pub(crate) fn operation(
        array: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
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
        registers.set(array.into(), result.into());
        Ok(())
    }
}

impl Operation for IteratorToArray {
    const NAME: &'static str = "IteratorToArray";
    const INSTRUCTION: &'static str = "INST - IteratorToArray";
    const COST: u8 = 8;
}

/// `IteratorStackEmpty` implements the Opcode Operation for `Opcode::IteratorStackEmpty`
///
/// Operation:
/// - Pushes `true` to the stack if the iterator stack is empty.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorStackEmpty;

impl IteratorStackEmpty {
    #[inline(always)]
    pub(crate) fn operation(
        empty: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let is_empty = context.vm.frame().iterators.is_empty();
        registers.set(empty.into(), is_empty.into());
    }
}

impl Operation for IteratorStackEmpty {
    const NAME: &'static str = "IteratorStackEmpty";
    const INSTRUCTION: &'static str = "INST - IteratorStackEmpty";
    const COST: u8 = 1;
}

/// `CreateIteratorResult` implements the Opcode Operation for `Opcode::CreateIteratorResult`
///
/// Operation:
/// -  Creates a new iterator result object
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateIteratorResult;

impl CreateIteratorResult {
    #[inline(always)]
    pub(crate) fn operation(
        (value, done): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let done = u32::from(done) != 0;
        let val = registers.get(value.into());
        let result = create_iter_result_object(val.clone(), done, context);
        registers.set(value.into(), result);
    }
}

impl Operation for CreateIteratorResult {
    const NAME: &'static str = "CreateIteratorResult";
    const INSTRUCTION: &'static str = "INST - CreateIteratorResult";
    const COST: u8 = 3;
}
