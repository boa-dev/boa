use crate::{
    builtins::Array,
    object::ObjectData,
    string::utf16,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

/// `PushNewArray` implements the Opcode Operation for `Opcode::PushNewArray`
///
/// Operation:
///  - Push an empty array value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushNewArray;

impl Operation for PushNewArray {
    const NAME: &'static str = "PushNewArray";
    const INSTRUCTION: &'static str = "INST - PushNewArray";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let array = context
            .intrinsics()
            .templates()
            .array()
            .create(ObjectData::array(), vec![JsValue::new(0)]);
        context.vm.push(array);
        Ok(CompletionType::Normal)
    }
}

/// `PushValueToArray` implements the Opcode Operation for `Opcode::PushValueToArray`
///
/// Operation:
///  - Push a value to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushValueToArray;

impl Operation for PushValueToArray {
    const NAME: &'static str = "PushValueToArray";
    const INSTRUCTION: &'static str = "INST - PushValueToArray";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let value = raw_context.vm.pop();
        let array = raw_context.vm.pop();
        let o = array.as_object().expect("should be an object");
        let len = o
            .length_of_array_like(context)
            .expect("should have 'length' property");
        o.create_data_property_or_throw(len, value, context)
            .expect("should be able to create new data property");
        context.as_raw_context_mut().vm.push(array);
        Ok(CompletionType::Normal)
    }
}

/// `PushEllisionToArray` implements the Opcode Operation for `Opcode::PushEllisionToArray`
///
/// Operation:
///  - Push an empty element/hole to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushElisionToArray;

impl Operation for PushElisionToArray {
    const NAME: &'static str = "PushElisionToArray";
    const INSTRUCTION: &'static str = "INST - PushElisionToArray";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let array = context.as_raw_context_mut().vm.pop();
        let o = array.as_object().expect("should always be an object");

        let len = o
            .length_of_array_like(context)
            .expect("arrays should always have a 'length' property");

        o.set(utf16!("length"), len + 1, true, context)?;
        context.as_raw_context_mut().vm.push(array);
        Ok(CompletionType::Normal)
    }
}

/// `PushIteratorToArray` implements the Opcode Operation for `Opcode::PushIteratorToArray`
///
/// Operation:
///  - Push all iterator values to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushIteratorToArray;

impl Operation for PushIteratorToArray {
    const NAME: &'static str = "PushIteratorToArray";
    const INSTRUCTION: &'static str = "INST - PushIteratorToArray";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let mut iterator = raw_context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");
        let array = raw_context.vm.pop();

        while !iterator.step(context)? {
            let next = iterator.value(context)?;
            Array::push(&array, &[next], context)?;
        }

        context.as_raw_context_mut().vm.push(array);
        Ok(CompletionType::Normal)
    }
}
