use crate::{
    property::PropertyKey,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `GetPropertyByName` implements the Opcode Operation for `Opcode::GetPropertyByName`
///
/// Operation:
///  - Get a property by name from an object an push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPropertyByName;

impl Operation for GetPropertyByName {
    const NAME: &'static str = "GetPropertyByName";
    const INSTRUCTION: &'static str = "INST - GetPropertyByName";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();

        let value = context.vm.pop();
        let object = if let Some(object) = value.as_object() {
            object.clone()
        } else {
            value.to_object(context)?
        };

        let name = context.vm.frame().code_block.names[index as usize];
        let key: PropertyKey = context.interner().resolve_expect(name.sym()).utf16().into();
        let result = object.__get__(&key, value, context)?;

        context.vm.push(result);
        Ok(ShouldExit::False)
    }
}

/// `GetPropertyByValue` implements the Opcode Operation for `Opcode::GetPropertyByValue`
///
/// Operation:
///  - Get a property by value from an object an push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPropertyByValue;

impl Operation for GetPropertyByValue {
    const NAME: &'static str = "GetPropertyByValue";
    const INSTRUCTION: &'static str = "INST - GetPropertyByValue";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let key = context.vm.pop();
        let value = context.vm.pop();
        let object = if let Some(object) = value.as_object() {
            object.clone()
        } else {
            value.to_object(context)?
        };

        let key = key.to_property_key(context)?;
        let result = object.__get__(&key, value, context)?;

        context.vm.push(result);
        Ok(ShouldExit::False)
    }
}

/// `GetPropertyByValuePush` implements the Opcode Operation for `Opcode::GetPropertyByValuePush`
///
/// Operation:
///  - Get a property by value from an object an push the key and value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPropertyByValuePush;

impl Operation for GetPropertyByValuePush {
    const NAME: &'static str = "GetPropertyByValuePush";
    const INSTRUCTION: &'static str = "INST - GetPropertyByValuePush";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let key = context.vm.pop();
        let value = context.vm.pop();
        let object = if let Some(object) = value.as_object() {
            object.clone()
        } else {
            value.to_object(context)?
        };

        let key = key.to_property_key(context)?;
        let result = object.__get__(&key, value, context)?;

        context.vm.push(key);
        context.vm.push(result);
        Ok(ShouldExit::False)
    }
}
