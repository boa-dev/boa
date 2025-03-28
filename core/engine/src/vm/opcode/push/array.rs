use crate::{
    builtins::Array,
    string::StaticJsStrings,
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionType, Registers,
    },
    Context, JsResult, JsValue,
};

/// `PushNewArray` implements the Opcode Operation for `Opcode::PushNewArray`
///
/// Operation:
///  - Push an empty array value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushNewArray;

impl PushNewArray {
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn operation(
        array: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = context
            .intrinsics()
            .templates()
            .array()
            .create(Array, Vec::from([JsValue::new(0)]));
        registers.set(array.into(), value.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushNewArray {
    const NAME: &'static str = "PushNewArray";
    const INSTRUCTION: &'static str = "INST - PushNewArray";
    const COST: u8 = 3;
}

/// `PushValueToArray` implements the Opcode Operation for `Opcode::PushValueToArray`
///
/// Operation:
///  - Push a value to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushValueToArray;

impl PushValueToArray {
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn operation(
        (value, array): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value.into());
        let array = registers.get(array.into());
        let o = array.as_object().expect("should be an object");
        let len = o
            .length_of_array_like(context)
            .expect("should have 'length' property");
        o.create_data_property_or_throw(len, value.clone(), context)
            .expect("should be able to create new data property");
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushValueToArray {
    const NAME: &'static str = "PushValueToArray";
    const INSTRUCTION: &'static str = "INST - PushValueToArray";
    const COST: u8 = 3;
}

/// `PushEllisionToArray` implements the Opcode Operation for `Opcode::PushEllisionToArray`
///
/// Operation:
///  - Push an empty element/hole to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushElisionToArray;

impl PushElisionToArray {
    pub(crate) fn operation(
        array: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let array = registers.get(array.into());
        let o = array.as_object().expect("should always be an object");
        let len = o
            .length_of_array_like(context)
            .expect("arrays should always have a 'length' property");
        o.set(StaticJsStrings::LENGTH, len + 1, true, context)?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushElisionToArray {
    const NAME: &'static str = "PushElisionToArray";
    const INSTRUCTION: &'static str = "INST - PushElisionToArray";
    const COST: u8 = 3;
}

/// `PushIteratorToArray` implements the Opcode Operation for `Opcode::PushIteratorToArray`
///
/// Operation:
///  - Push all iterator values to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushIteratorToArray;

impl PushIteratorToArray {
    pub(crate) fn operation(
        array: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let array = registers.get(array.into());
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .expect("iterator stack should have at least an iterator");
        while let Some(next) = iterator.step_value(context)? {
            Array::push(array, &[next], context)?;
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushIteratorToArray {
    const NAME: &'static str = "PushIteratorToArray";
    const INSTRUCTION: &'static str = "INST - PushIteratorToArray";
    const COST: u8 = 8;
}
