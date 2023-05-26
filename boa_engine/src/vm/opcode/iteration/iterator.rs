use crate::{
    builtins::{
        iterable::{IteratorRecord, IteratorResult},
        Array,
    },
    vm::{opcode::Operation, CompletionType},
    Context, JsNativeError, JsResult,
};

/// `IteratorNext` implements the Opcode Operation for `Opcode::IteratorNext`
///
/// Operation:
///  - Calls the `next` method of `iterator` and puts its return value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorNext;

impl Operation for IteratorNext {
    const NAME: &'static str = "IteratorNext";
    const INSTRUCTION: &'static str = "INST - IteratorNext";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let next_result = next_method.call(&iterator, &[], context)?;
        context.vm.push(iterator);
        context.vm.push(next_method);
        context.vm.push(next_result);
        Ok(CompletionType::Normal)
    }
}

/// `IteratorNextSetDone` implements the Opcode Operation for `Opcode::IteratorNextSetDone`
///
/// Operation:
///  - Calls the `next` method of `iterator`, puts its return value on the stack
///    and sets the `[[Done]]` value of the iterator on the call frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorNextSetDone;

impl Operation for IteratorNextSetDone {
    const NAME: &'static str = "IteratorNextSetDone";
    const INSTRUCTION: &'static str = "INST - IteratorNextSetDone";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let mut done = true;
        let result = next_method
            .call(&iterator, &[], context)
            .and_then(|next_result| {
                next_method
                    .as_object()
                    .cloned()
                    .map(IteratorResult::new)
                    .ok_or_else(|| {
                        JsNativeError::typ()
                            .with_message("next value should be an object")
                            .into()
                    })
                    .and_then(|iterator_result| {
                        iterator_result.complete(context).map(|d| {
                            done = d;
                            context.vm.push(iterator);
                            context.vm.push(next_method);
                            context.vm.push(next_result);
                            CompletionType::Normal
                        })
                    })
            });

        context
            .vm
            .frame_mut()
            .iterators
            .last_mut()
            .expect("iterator on the call frame must exist")
            .1 = done;

        result
    }
}

/// `IteratorUnwrapNext` implements the Opcode Operation for `Opcode::IteratorUnwrapNext`
///
/// Operation:
///  - Gets the `value` and `done` properties of an iterator result.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorUnwrapNext;

impl Operation for IteratorUnwrapNext {
    const NAME: &'static str = "IteratorUnwrapNext";
    const INSTRUCTION: &'static str = "INST - IteratorUnwrapNext";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let next_result = context.vm.pop();
        let next_result = next_result
            .as_object()
            .cloned()
            .map(IteratorResult::new)
            .ok_or_else(|| JsNativeError::typ().with_message("next value should be an object"))?;
        let complete = next_result.complete(context)?;
        let value = next_result.value(context)?;
        context.vm.push(complete);
        context.vm.push(value);

        Ok(CompletionType::Normal)
    }
}

/// `IteratorUnwrapValue` implements the Opcode Operation for `Opcode::IteratorUnwrapValue`
///
/// Operation:
///  - Gets the `value` property of an iterator result.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorUnwrapValue;

impl Operation for IteratorUnwrapValue {
    const NAME: &'static str = "IteratorUnwrapValue";
    const INSTRUCTION: &'static str = "INST - IteratorUnwrapValue";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let next_result = context.vm.pop();
        let next_result = next_result
            .as_object()
            .cloned()
            .map(IteratorResult::new)
            .ok_or_else(|| JsNativeError::typ().with_message("next value should be an object"))?;
        let value = next_result.value(context)?;
        context.vm.push(value);

        Ok(CompletionType::Normal)
    }
}

/// `IteratorUnwrapNextOrJump` implements the Opcode Operation for `Opcode::IteratorUnwrapNextOrJump`
///
/// Operation:
///  - Gets the `value` and `done` properties of an iterator result, or jump to `address` if
///    `done` is true.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorUnwrapNextOrJump;

impl Operation for IteratorUnwrapNextOrJump {
    const NAME: &'static str = "IteratorUnwrapNextOrJump";
    const INSTRUCTION: &'static str = "INST - IteratorUnwrapNextOrJump";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();

        let next_result = context.vm.pop();
        let next_result = next_result
            .as_object()
            .cloned()
            .map(IteratorResult::new)
            .ok_or_else(|| JsNativeError::typ().with_message("next value should be an object"))?;

        if next_result.complete(context)? {
            context.vm.frame_mut().pc = address;
            context.vm.push(true);
        } else {
            context.vm.push(false);
            let value = next_result.value(context)?;
            context.vm.push(value);
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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let iterator = iterator.as_object().expect("iterator was not an object");

        let iterator_record = IteratorRecord::new(iterator.clone(), next_method.clone(), false);
        let mut values = Vec::new();

        let err = loop {
            match iterator_record.step(context) {
                Ok(Some(result)) => match result.value(context) {
                    Ok(value) => values.push(value),
                    Err(err) => break Some(err),
                },
                Ok(None) => break None,
                Err(err) => break Some(err),
            }
        };

        context
            .vm
            .frame_mut()
            .iterators
            .last_mut()
            .expect("should exist")
            .1 = true;
        if let Some(err) = err {
            return Err(err);
        }

        let array = Array::create_array_from_list(values, context);

        context.vm.push(iterator.clone());
        context.vm.push(next_method);
        context.vm.push(array);
        Ok(CompletionType::Normal)
    }
}

/// `IteratorClosePush` implements the Opcode Operation for `Opcode::IteratorClosePush`
///
/// Operation:
///  - Push an iterator to the call frame close iterator stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorClosePush;

impl Operation for IteratorClosePush {
    const NAME: &'static str = "IteratorClosePush";
    const INSTRUCTION: &'static str = "INST - IteratorClosePush";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let iterator_object = iterator.as_object().expect("iterator was not an object");
        context
            .vm
            .frame_mut()
            .iterators
            .push((iterator_object.clone(), false));
        context.vm.push(iterator);
        context.vm.push(next_method);
        Ok(CompletionType::Normal)
    }
}

/// `IteratorClosePop` implements the Opcode Operation for `Opcode::IteratorClosePop`
///
/// Operation:
///  - Pop an iterator from the call frame close iterator stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorClosePop;

impl Operation for IteratorClosePop {
    const NAME: &'static str = "IteratorClosePop";
    const INSTRUCTION: &'static str = "INST - IteratorClosePop";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        context.vm.frame_mut().iterators.pop();
        Ok(CompletionType::Normal)
    }
}
