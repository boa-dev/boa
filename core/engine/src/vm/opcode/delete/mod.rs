use crate::{
    error::JsNativeError,
    object::internal_methods::InternalMethodContext,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

use super::VaryingOperand;

/// `DeletePropertyByName` implements the Opcode Operation for `Opcode::DeletePropertyByName`
///
/// Operation:
///  - Deletes a property by name of an object
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeletePropertyByName;

impl DeletePropertyByName {
    #[inline(always)]
    pub(super) fn operation(
        (object_register, index): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(object_register.into());
        let object = object.to_object(context)?;
        let code_block = context.vm.frame().code_block();
        let key = code_block.constant_string(index.into()).into();
        let strict = code_block.strict();

        let result = object.__delete__(&key, &mut InternalMethodContext::new(context))?;
        if !result && strict {
            return Err(JsNativeError::typ()
                .with_message("Cannot delete property")
                .into());
        }
        registers.set(object_register.into(), result.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for DeletePropertyByName {
    const NAME: &'static str = "DeletePropertyByName";
    const INSTRUCTION: &'static str = "INST - DeletePropertyByName";
    const COST: u8 = 3;
}

/// `DeletePropertyByValue` implements the Opcode Operation for `Opcode::DeletePropertyByValue`
///
/// Operation:
///  - Deletes a property by value of an object
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeletePropertyByValue;

impl DeletePropertyByValue {
    #[inline(always)]
    pub(super) fn operation(
        (object_register, key): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(object_register.into());
        let key = registers.get(key.into());
        let object = object.to_object(context)?;
        let property_key = key.to_property_key(context)?;

        let result = object.__delete__(&property_key, &mut InternalMethodContext::new(context))?;
        if !result && context.vm.frame().code_block().strict() {
            return Err(JsNativeError::typ()
                .with_message("Cannot delete property")
                .into());
        }
        registers.set(object_register.into(), result.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for DeletePropertyByValue {
    const NAME: &'static str = "DeletePropertyByValue";
    const INSTRUCTION: &'static str = "INST - DeletePropertyByValue";
    const COST: u8 = 3;
}

/// `DeleteName` implements the Opcode Operation for `Opcode::DeleteName`
///
/// Operation:
///  - Deletes a property by value of an object
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeleteName;

impl DeleteName {
    #[inline(always)]
    pub(super) fn operation(
        (value, index): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let mut binding_locator =
            context.vm.frame().code_block.bindings[usize::from(index)].clone();
        context.find_runtime_binding(&mut binding_locator)?;
        let deleted = context.delete_binding(&binding_locator)?;
        registers.set(value.into(), deleted.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for DeleteName {
    const NAME: &'static str = "DeleteName";
    const INSTRUCTION: &'static str = "INST - DeleteName";
    const COST: u8 = 3;
}

/// `DeleteSuperThrow` implements the Opcode Operation for `Opcode::DeleteSuperThrow`
///
/// Operation:
///  - Throws an error when trying to delete a property of `super`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeleteSuperThrow;

impl DeleteSuperThrow {
    #[inline(always)]
    pub(super) fn operation(_: (), _: &mut Registers, _: &mut Context) -> JsResult<CompletionType> {
        Err(JsNativeError::reference()
            .with_message("cannot delete a property of `super`")
            .into())
    }
}

impl Operation for DeleteSuperThrow {
    const NAME: &'static str = "DeleteSuperThrow";
    const INSTRUCTION: &'static str = "INST - DeleteSuperThrow";
    const COST: u8 = 2;
}
