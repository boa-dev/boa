use crate::{
    Context, JsNativeError, JsResult,
    object::internal_methods::InternalMethodPropertyContext,
    property::PropertyDescriptor,
    vm::opcode::{Operation, VaryingOperand},
};

/// `DefineOwnPropertyByName` implements the Opcode Operation for `Opcode::DefineOwnPropertyByName`
///
/// Operation:
///  - Defines a own property of an object by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineOwnPropertyByName;

impl DefineOwnPropertyByName {
    #[inline(always)]
    pub(crate) fn operation(
        (object, value, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &Context,
    ) -> JsResult<()> {
        let object = context.get_register(object.into()).clone();
        let value = context.get_register(value.into()).clone();
        let name = context.with_vm(|vm| vm.frame().code_block().constant_string(index.into()));
        let object = object.to_object(context)?;
        object.__define_own_property__(
            &name.into(),
            PropertyDescriptor::builder()
                .value(value.clone())
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .build(),
            &mut InternalMethodPropertyContext::new(context),
        )?;
        Ok(())
    }
}

impl Operation for DefineOwnPropertyByName {
    const NAME: &'static str = "DefineOwnPropertyByName";
    const INSTRUCTION: &'static str = "INST - DefineOwnPropertyByName";
    const COST: u8 = 4;
}

/// `DefineOwnPropertyByValue` implements the Opcode Operation for `Opcode::DefineOwnPropertyByValue`
///
/// Operation:
///  - Defines a own property of an object by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineOwnPropertyByValue;

impl DefineOwnPropertyByValue {
    #[inline(always)]
    pub(crate) fn operation(
        (value, key, object): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &Context,
    ) -> JsResult<()> {
        let value = context.get_register(value.into()).clone();
        let key = context.get_register(key.into()).clone();
        let object = context.get_register(object.into()).clone();
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
            &mut InternalMethodPropertyContext::new(context),
        )?;
        if !success {
            return Err(JsNativeError::typ()
                .with_message("failed to defined own property")
                .into());
        }
        Ok(())
    }
}

impl Operation for DefineOwnPropertyByValue {
    const NAME: &'static str = "DefineOwnPropertyByValue";
    const INSTRUCTION: &'static str = "INST - DefineOwnPropertyByValue";
    const COST: u8 = 4;
}
