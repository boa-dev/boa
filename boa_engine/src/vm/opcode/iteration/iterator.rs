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
            context.vm.frame_mut().pc = address as usize;
            context.vm.frame_mut().dec_frame_env_stack();
            context.vm.environments.pop();
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

        while let Some(result) = iterator_record.step(context)? {
            values.push(result.value(context)?);
        }

        let array = Array::create_array_from_list(values, context);

        context.vm.push(iterator.clone());
        context.vm.push(next_method);
        context.vm.push(array);
        Ok(CompletionType::Normal)
    }
}
