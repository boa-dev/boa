use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `DeletePropertyByName` implements the Opcode Operation for `Opcode::DeletePropertyByName`
///
/// Operation:
///  - Deletes a property by name of an object
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeletePropertyByName;

impl Operation for DeletePropertyByName {
    const NAME: &'static str = "DeletePropertyByName";
    const INSTRUCTION: &'static str = "INST - DeletePropertyByName";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let value = context.vm.pop();
        let object = value.to_object(context)?;
        let key = context.vm.frame().code_block.names[index as usize]
            .clone()
            .into();
        let result = object.__delete__(&key, context)?;
        if !result && context.vm.frame().code_block.strict() {
            return Err(JsNativeError::typ()
                .with_message("Cannot delete property")
                .into());
        }
        context.vm.push(result);
        Ok(CompletionType::Normal)
    }
}

/// `DeletePropertyByValue` implements the Opcode Operation for `Opcode::DeletePropertyByValue`
///
/// Operation:
///  - Deletes a property by value of an object
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeletePropertyByValue;

impl Operation for DeletePropertyByValue {
    const NAME: &'static str = "DeletePropertyByValue";
    const INSTRUCTION: &'static str = "INST - DeletePropertyByValue";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let key_value = context.vm.pop();
        let value = context.vm.pop();
        let object = value.to_object(context)?;
        let property_key = key_value.to_property_key(context)?;
        let result = object.__delete__(&property_key, context)?;
        if !result && context.vm.frame().code_block.strict() {
            return Err(JsNativeError::typ()
                .with_message("Cannot delete property")
                .into());
        }
        context.vm.push(result);
        Ok(CompletionType::Normal)
    }
}

/// `DeleteName` implements the Opcode Operation for `Opcode::DeleteName`
///
/// Operation:
///  - Deletes a property by value of an object
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeleteName;

impl Operation for DeleteName {
    const NAME: &'static str = "DeleteName";
    const INSTRUCTION: &'static str = "INST - DeleteName";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let mut binding_locator = context.vm.frame().code_block.bindings[index as usize];

        context.find_runtime_binding(&mut binding_locator)?;

        let deleted = context.delete_binding(binding_locator)?;

        context.vm.push(deleted);
        Ok(CompletionType::Normal)
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

    fn execute(_: &mut Context<'_>) -> JsResult<CompletionType> {
        Err(JsNativeError::reference()
            .with_message("cannot delete a property of `super`")
            .into())
    }
}
