use std::matches;

use crate::{
    builtins::{iterable::create_iter_result_object, Array},
    js_string,
    vm::{opcode::Operation, CompletionType, GeneratorResumeKind},
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let mut iterator = context
            .as_raw_context_mut()
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");

        iterator.step(context)?;

        context
            .as_raw_context_mut()
            .vm
            .frame_mut()
            .iterators
            .push(iterator);

        Ok(CompletionType::Normal)
    }
}

/// `IteratorFinishAsyncNext` implements the Opcode Operation for `Opcode::IteratorFinishAsyncNext`.
///
/// Operation:
///  - Finishes the call to `Opcode::IteratorNext` within a `for await` loop by setting the current
/// result of the current iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorFinishAsyncNext;

impl Operation for IteratorFinishAsyncNext {
    const NAME: &'static str = "IteratorFinishAsyncNext";
    const INSTRUCTION: &'static str = "INST - IteratorFinishAsyncNext";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let mut iterator = raw_context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        if matches!(
            raw_context.vm.frame().generator_resume_kind,
            GeneratorResumeKind::Throw
        ) {
            // If after awaiting the `next` call the iterator returned an error, it can be considered
            // as poisoned, meaning we can remove it from the iterator stack to avoid calling
            // cleanup operations on it.
            return Ok(CompletionType::Normal);
        }

        let next_result = raw_context.vm.pop();

        iterator.update_result(next_result, context)?;

        context
            .as_raw_context_mut()
            .vm
            .frame_mut()
            .iterators
            .push(iterator);
        Ok(CompletionType::Normal)
    }
}

/// `IteratorResult` implements the Opcode Operation for `Opcode::IteratorResult`
///
/// Operation:
///  - Gets the last iteration result of the current iterator record.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorResult;

impl Operation for IteratorResult {
    const NAME: &'static str = "IteratorResult";
    const INSTRUCTION: &'static str = "INST - IteratorResult";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let last_result = context
            .vm
            .frame()
            .iterators
            .last()
            .expect("iterator on the call frame must exist")
            .last_result()
            .object()
            .clone();

        context.vm.push(last_result);

        Ok(CompletionType::Normal)
    }
}

/// `IteratorValue` implements the Opcode Operation for `Opcode::IteratorValue`
///
/// Operation:
///  - Gets the `value` property of the current iterator record.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorValue;

impl Operation for IteratorValue {
    const NAME: &'static str = "IteratorValue";
    const INSTRUCTION: &'static str = "INST - IteratorValue";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let mut iterator = context
            .as_raw_context_mut()
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        let value = iterator.value(context)?;
        {
            let context = context.as_raw_context_mut();
            context.vm.push(value);

            context.vm.frame_mut().iterators.push(iterator);
        }

        Ok(CompletionType::Normal)
    }
}

/// `IteratorDone` implements the Opcode Operation for `Opcode::IteratorDone`
///
/// Operation:
///  - Returns `true` if the current iterator is done, or `false` otherwise
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorDone;

impl Operation for IteratorDone {
    const NAME: &'static str = "IteratorDone";
    const INSTRUCTION: &'static str = "INST - IteratorDone";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let done = context
            .vm
            .frame()
            .iterators
            .last()
            .expect("iterator on the call frame must exist")
            .done();

        context.vm.push(done);

        Ok(CompletionType::Normal)
    }
}

/// `IteratorReturn` implements the Opcode Operation for `Opcode::IteratorReturn`
///
/// Operation:
///  - Calls `return` on the current iterator and returns the result.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorReturn;

impl Operation for IteratorReturn {
    const NAME: &'static str = "IteratorReturn";
    const INSTRUCTION: &'static str = "INST - IteratorReturn";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let record = context
            .as_raw_context_mut()
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator on the call frame must exist");

        let Some(ret) = record.iterator().get_method(js_string!("return"), context)? else {
            context.as_raw_context_mut().vm.push(false);
            return Ok(CompletionType::Normal);
        };

        let value = ret.call(&record.iterator().clone().into(), &[], context)?;
        {
            let context = context.as_raw_context_mut();
            context.vm.push(value);
            context.vm.push(true);
        }

        Ok(CompletionType::Normal)
    }
}

/// `IteratorToArray` implements the Opcode Operation for `Opcode::IteratorToArray`
///
/// Operation:
///  - Consume the iterator and construct and array with all the values.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorToArray;

impl Operation for IteratorToArray {
    const NAME: &'static str = "IteratorToArray";
    const INSTRUCTION: &'static str = "INST - IteratorToArray";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let mut iterator = context
            .as_raw_context_mut()
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
                    context
                        .as_raw_context_mut()
                        .vm
                        .frame_mut()
                        .iterators
                        .push(iterator);
                    return Err(err);
                }
            };

            if done {
                break;
            }

            match iterator.value(context) {
                Ok(value) => values.push(value),
                Err(err) => {
                    context
                        .as_raw_context_mut()
                        .vm
                        .frame_mut()
                        .iterators
                        .push(iterator);
                    return Err(err);
                }
            }
        }

        {
            let context = context.as_raw_context_mut();

            context.vm.frame_mut().iterators.push(iterator);

            let array = Array::create_array_from_list(values, context);

            context.vm.push(array);
        }

        Ok(CompletionType::Normal)
    }
}

/// `IteratorPop` implements the Opcode Operation for `Opcode::IteratorPop`
///
/// Operation:
///  - Pop an iterator from the call frame close iterator stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorPop;

impl Operation for IteratorPop {
    const NAME: &'static str = "IteratorPop";
    const INSTRUCTION: &'static str = "INST - IteratorPop";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        context.vm.frame_mut().iterators.pop();
        Ok(CompletionType::Normal)
    }
}

/// `IteratorStackEmpty` implements the Opcode Operation for `Opcode::IteratorStackEmpty`
///
/// Operation:
/// - Pushes `true` to the stack if the iterator stack is empty.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorStackEmpty;

impl Operation for IteratorStackEmpty {
    const NAME: &'static str = "IteratorStackEmpty";
    const INSTRUCTION: &'static str = "INST - IteratorStackEmpty";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let is_empty = context.vm.frame().iterators.is_empty();
        context.vm.push(is_empty);
        Ok(CompletionType::Normal)
    }
}

/// `CreateIteratorResult` implements the Opcode Operation for `Opcode::CreateIteratorResult`
///
/// Operation:
/// -  Creates a new iterator result object
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateIteratorResult;

impl Operation for CreateIteratorResult {
    const NAME: &'static str = "CreateIteratorResult";
    const INSTRUCTION: &'static str = "INST - CreateIteratorResult";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let value = raw_context.vm.pop();
        let done = raw_context.vm.read::<u8>() != 0;

        let result = create_iter_result_object(value, done, context);

        context.as_raw_context_mut().vm.push(result);

        Ok(CompletionType::Normal)
    }
}
