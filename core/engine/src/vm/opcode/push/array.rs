use crate::{
    builtins::Array,
    string::StaticJsStrings,
    vm::{opcode::Operation, CompletionType, Registers},
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
    fn operation(
        array: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = context
            .intrinsics()
            .templates()
            .array()
            .create(Array, Vec::from([JsValue::new(0)]));
        registers.set(array, value.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushNewArray {
    const NAME: &'static str = "PushNewArray";
    const INSTRUCTION: &'static str = "INST - PushNewArray";
    const COST: u8 = 3;

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

/// `PushValueToArray` implements the Opcode Operation for `Opcode::PushValueToArray`
///
/// Operation:
///  - Push a value to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushValueToArray;

impl PushValueToArray {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        value: u32,
        array: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        let array = registers.get(array);
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

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let array = context.vm.read::<u8>().into();
        Self::operation(value, array, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let array = context.vm.read::<u16>().into();
        Self::operation(value, array, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let array = context.vm.read::<u32>();
        Self::operation(value, array, registers, context)
    }
}

/// `PushEllisionToArray` implements the Opcode Operation for `Opcode::PushEllisionToArray`
///
/// Operation:
///  - Push an empty element/hole to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushElisionToArray;

impl PushElisionToArray {
    fn operation(
        array: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let array = registers.get(array);
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

/// `PushIteratorToArray` implements the Opcode Operation for `Opcode::PushIteratorToArray`
///
/// Operation:
///  - Push all iterator values to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushIteratorToArray;

impl PushIteratorToArray {
    fn operation(
        array: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let array = registers.get(array);
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
