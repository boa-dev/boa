use crate::{
    property::PropertyDescriptor,
    vm::{opcode::Operation, CompletionType},
    Context, JsNativeError, JsResult,
};

/// `DefineOwnPropertyByName` implements the Opcode Operation for `Opcode::DefineOwnPropertyByName`
///
/// Operation:
///  - Defines a own property of an object by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineOwnPropertyByName;

impl Operation for DefineOwnPropertyByName {
    const NAME: &'static str = "DefineOwnPropertyByName";
    const INSTRUCTION: &'static str = "INST - DefineOwnPropertyByName";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let index = raw_context.vm.read::<u32>();
        let value = raw_context.vm.pop();
        let object = raw_context.vm.pop();
        let name = raw_context.vm.frame().code_block.names[index as usize].clone();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            object.to_object(context)?
        };
        object.__define_own_property__(
            &name.into(),
            PropertyDescriptor::builder()
                .value(value)
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(CompletionType::Normal)
    }
}

/// `DefineOwnPropertyByValue` implements the Opcode Operation for `Opcode::DefineOwnPropertyByValue`
///
/// Operation:
///  - Defines a own property of an object by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineOwnPropertyByValue;

impl Operation for DefineOwnPropertyByValue {
    const NAME: &'static str = "DefineOwnPropertyByValue";
    const INSTRUCTION: &'static str = "INST - DefineOwnPropertyByValue";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let value = raw_context.vm.pop();
        let key = raw_context.vm.pop();
        let object = raw_context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            object.to_object(context)?
        };
        let key = key.to_property_key(context)?;
        let success = object.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .value(value)
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .build(),
            context,
        )?;
        if !success {
            return Err(JsNativeError::typ()
                .with_message("failed to defined own property")
                .into());
        }
        Ok(CompletionType::Normal)
    }
}
