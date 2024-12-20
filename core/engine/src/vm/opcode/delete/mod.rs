use crate::{
    error::JsNativeError,
    object::internal_methods::InternalMethodContext,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `DeletePropertyByName` implements the Opcode Operation for `Opcode::DeletePropertyByName`
///
/// Operation:
///  - Deletes a property by name of an object
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeletePropertyByName;

impl DeletePropertyByName {
    fn operation(
        object_register: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(object_register);
        let object = object.to_object(context)?;
        let code_block = context.vm.frame().code_block();
        let key = code_block.constant_string(index).into();
        let strict = code_block.strict();

        let result = object.__delete__(&key, &mut InternalMethodContext::new(context))?;
        if !result && strict {
            return Err(JsNativeError::typ()
                .with_message("Cannot delete property")
                .into());
        }
        registers.set(object_register, result.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for DeletePropertyByName {
    const NAME: &'static str = "DeletePropertyByName";
    const INSTRUCTION: &'static str = "INST - DeletePropertyByName";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(object, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(object, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(object, index, registers, context)
    }
}

/// `DeletePropertyByValue` implements the Opcode Operation for `Opcode::DeletePropertyByValue`
///
/// Operation:
///  - Deletes a property by value of an object
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeletePropertyByValue;

impl DeletePropertyByValue {
    fn operation(
        object_register: u32,
        key: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(object_register);
        let key = registers.get(key);
        let object = object.to_object(context)?;
        let property_key = key.to_property_key(context)?;

        let result = object.__delete__(&property_key, &mut InternalMethodContext::new(context))?;
        if !result && context.vm.frame().code_block().strict() {
            return Err(JsNativeError::typ()
                .with_message("Cannot delete property")
                .into());
        }
        registers.set(object_register, result.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for DeletePropertyByValue {
    const NAME: &'static str = "DeletePropertyByValue";
    const INSTRUCTION: &'static str = "INST - DeletePropertyByValue";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u8>().into();
        let key = context.vm.read::<u8>().into();
        Self::operation(object, key, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u16>().into();
        let key = context.vm.read::<u16>().into();
        Self::operation(object, key, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u32>();
        let key = context.vm.read::<u32>();
        Self::operation(object, key, registers, context)
    }
}

/// `DeleteName` implements the Opcode Operation for `Opcode::DeleteName`
///
/// Operation:
///  - Deletes a property by value of an object
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeleteName;

impl DeleteName {
    fn operation(
        value: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let mut binding_locator = context.vm.frame().code_block.bindings[index].clone();
        context.find_runtime_binding(&mut binding_locator)?;
        let deleted = context.delete_binding(&binding_locator)?;
        registers.set(value, deleted.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for DeleteName {
    const NAME: &'static str = "DeleteName";
    const INSTRUCTION: &'static str = "INST - DeleteName";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(value, index, registers, context)
    }
}

/// `DeleteSuperThrow` implements the Opcode Operation for `Opcode::DeleteSuperThrow`
///
/// Operation:
///  - Throws an error when trying to delete a property of `super`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeleteSuperThrow;

impl Operation for DeleteSuperThrow {
    const NAME: &'static str = "DeleteSuperThrow";
    const INSTRUCTION: &'static str = "INST - DeleteSuperThrow";
    const COST: u8 = 2;

    fn execute(_: &mut Registers, _: &mut Context) -> JsResult<CompletionType> {
        Err(JsNativeError::reference()
            .with_message("cannot delete a property of `super`")
            .into())
    }
}
