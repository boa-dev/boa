use crate::{
    object::internal_methods::InternalMethodContext,
    property::PropertyDescriptor,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsNativeError, JsResult,
};

/// `DefineOwnPropertyByName` implements the Opcode Operation for `Opcode::DefineOwnPropertyByName`
///
/// Operation:
///  - Defines a own property of an object by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineOwnPropertyByName;

impl DefineOwnPropertyByName {
    fn operation(
        object: u32,
        value: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(object);
        let value = registers.get(value);
        let name = context.vm.frame().code_block().constant_string(index);
        let object = object.to_object(context)?;
        object.__define_own_property__(
            &name.into(),
            PropertyDescriptor::builder()
                .value(value.clone())
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineOwnPropertyByName {
    const NAME: &'static str = "DefineOwnPropertyByName";
    const INSTRUCTION: &'static str = "INST - DefineOwnPropertyByName";
    const COST: u8 = 4;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u8>().into();
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(object, value, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u16>().into();
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(object, value, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(object, value, index, registers, context)
    }
}

/// `DefineOwnPropertyByValue` implements the Opcode Operation for `Opcode::DefineOwnPropertyByValue`
///
/// Operation:
///  - Defines a own property of an object by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineOwnPropertyByValue;

impl DefineOwnPropertyByValue {
    fn operation(
        value: u32,
        key: u32,
        object: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        let key = registers.get(key);
        let object = registers.get(object);
        let object = object.to_object(context)?;
        let key = key.to_property_key(context)?;
        let success = object.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .value(value.clone())
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        if !success {
            return Err(JsNativeError::typ()
                .with_message("failed to defined own property")
                .into());
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineOwnPropertyByValue {
    const NAME: &'static str = "DefineOwnPropertyByValue";
    const INSTRUCTION: &'static str = "INST - DefineOwnPropertyByValue";
    const COST: u8 = 4;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let key = context.vm.read::<u8>().into();
        let object = context.vm.read::<u8>().into();
        Self::operation(value, key, object, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let key = context.vm.read::<u16>().into();
        let object = context.vm.read::<u16>().into();
        Self::operation(value, key, object, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let key = context.vm.read::<u32>();
        let object = context.vm.read::<u32>();
        Self::operation(value, key, object, registers, context)
    }
}
