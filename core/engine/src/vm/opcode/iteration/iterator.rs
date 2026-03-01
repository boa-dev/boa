use crate::{
    Context, JsResult,
    builtins::{Array, iterable::create_iter_result_object},
    js_string,
    vm::{
        GeneratorResumeKind,
        opcode::{Operation, VaryingOperand},
    },
};

/// `IteratorNext` implements the Opcode Operation for `Opcode::IteratorNext`
///
/// Operation:
///  - Calls the `next` method of `iterator`, updating its record with the next value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorNext;

impl IteratorNext {
    #[inline(always)]
    pub(crate) fn operation((): (), context: &Context) -> JsResult<()> {
        let mut iterator = context
            .vm_mut()
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");

        iterator.step(context)?;

        context.vm_mut().frame_mut().iterators.push(iterator);

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
        context: &Context,
    ) -> JsResult<()> {
        let mut iterator = context
            .vm_mut()
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        let resume_kind = context
            .vm_mut()
            .get_register(resume_kind.into())
            .to_generator_resume_kind();

        if matches!(resume_kind, GeneratorResumeKind::Throw) {
            // If after awaiting the `next` call the iterator returned an error, it can be considered
            // as poisoned, meaning we can remove it from the iterator stack to avoid calling
            // cleanup operations on it.
            return Ok(());
        }

        let value = context.vm_mut().get_register(value.into()).clone();
        iterator.update_result(value, context)?;
        context.vm_mut().frame_mut().iterators.push(iterator);
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
    pub(crate) fn operation(value: VaryingOperand, context: &Context) {
        let last_result = context
            .vm_mut()
            .frame()
            .iterators
            .last()
            .expect("iterator on the call frame must exist")
            .last_result()
            .object()
            .clone();
        context
            .vm_mut()
            .set_register(value.into(), last_result.into());
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
    pub(crate) fn operation(value: VaryingOperand, context: &Context) -> JsResult<()> {
        let mut iterator = context
            .vm_mut()
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        let iter_value = iterator.value(context)?;
        context.vm_mut().set_register(value.into(), iter_value);

        context.vm_mut().frame_mut().iterators.push(iterator);

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
    pub(crate) fn operation(done: VaryingOperand, context: &Context) {
        let value = context
            .vm_mut()
            .frame()
            .iterators
            .last()
            .expect("iterator on the call frame must exist")
            .done();
        context.vm_mut().set_register(done.into(), value.into());
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
        context: &Context,
    ) -> JsResult<()> {
        let Some(record) = context.vm_mut().frame_mut().iterators.pop() else {
            context.vm_mut().set_register(called.into(), false.into());
            return Ok(());
        };

        if record.done() {
            context.vm_mut().set_register(called.into(), false.into());
            return Ok(());
        }

        let Some(ret) = record
            .iterator()
            .get_method(js_string!("return"), context)?
        else {
            context.vm_mut().set_register(called.into(), false.into());
            return Ok(());
        };

        let old_return_value = context.vm_mut().get_return_value();

        let return_value = ret.call(&record.iterator().clone().into(), &[], context)?;

        context.vm_mut().set_return_value(old_return_value);

        context.vm_mut().set_register(value.into(), return_value);
        context.vm_mut().set_register(called.into(), true.into());

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
    pub(crate) fn operation(array: VaryingOperand, context: &Context) -> JsResult<()> {
        let mut iterator = context
            .vm_mut()
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        let mut values = Vec::new();

        loop {
            let done = match iterator.step(context) {
                Ok(done) => done,
                Err(err) => {
                    context.vm_mut().frame_mut().iterators.push(iterator);
                    return Err(err);
                }
            };

            if done {
                break;
            }

            match iterator.value(context) {
                Ok(value) => values.push(value),
                Err(err) => {
                    context.vm_mut().frame_mut().iterators.push(iterator);
                    return Err(err);
                }
            }
        }

        context.vm_mut().frame_mut().iterators.push(iterator);
        let result = Array::create_array_from_list(values, context);
        context.vm_mut().set_register(array.into(), result.into());
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
    pub(crate) fn operation(empty: VaryingOperand, context: &Context) {
        let is_empty = context.vm_mut().frame().iterators.is_empty();
        context.vm_mut().set_register(empty.into(), is_empty.into());
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
    pub(crate) fn operation((value, done): (VaryingOperand, VaryingOperand), context: &Context) {
        let done = u32::from(done) != 0;
        let val = context.vm_mut().get_register(value.into()).clone();
        let result = create_iter_result_object(val, done, context);
        context.vm_mut().set_register(value.into(), result);
    }
}

impl Operation for CreateIteratorResult {
    const NAME: &'static str = "CreateIteratorResult";
    const INSTRUCTION: &'static str = "INST - CreateIteratorResult";
    const COST: u8 = 3;
}
