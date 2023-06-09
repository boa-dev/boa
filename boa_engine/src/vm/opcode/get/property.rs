use crate::{
    property::PropertyKey,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let index = raw_context.vm.read::<u32>();

        let value = raw_context.vm.pop();

        let key = raw_context.vm.frame().code_block.names[index as usize]
            .clone()
            .into();

        let object = if let Some(object) = value.as_object() {
            object.clone()
        } else {
            value.to_object(context)?
        };

        let result = object.__get__(&key, value, context)?;

        context.as_raw_context_mut().vm.push(result);
        Ok(CompletionType::Normal)
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let key = raw_context.vm.pop();
        let value = raw_context.vm.pop();
        let object = if let Some(object) = value.as_object() {
            object.clone()
        } else {
            value.to_object(context)?
        };

        let key = key.to_property_key(context)?;

        // Fast Path
        if object.is_array() {
            if let PropertyKey::Index(index) = &key {
                let object_borrowed = object.borrow();
                if let Some(element) = object_borrowed
                    .properties()
                    .dense_indexed_properties()
                    .and_then(|vec| vec.get(*index as usize))
                {
                    context.as_raw_context_mut().vm.push(element.clone());
                    return Ok(CompletionType::Normal);
                }
            }
        }

        // Slow path:
        let result = object.__get__(&key, value, context)?;

        context.as_raw_context_mut().vm.push(result);
        Ok(CompletionType::Normal)
    }
}

/// `GetMethod` implements the Opcode Operation for `Opcode::GetMethod`
///
/// Operation:
///  - Get a property method or undefined if the property is null or undefined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetMethod;

impl Operation for GetMethod {
    const NAME: &'static str = "GetMethod";
    const INSTRUCTION: &'static str = "INST - GetMethod";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let index = raw_context.vm.read::<u32>();
        let key = raw_context.vm.frame().code_block.names[index as usize].clone();
        let value = raw_context.vm.pop();

        let method = value.get_method(key, context)?;
        {
            let context = context.as_raw_context_mut();
            context.vm.push(value);
            context
                .vm
                .push(method.map(JsValue::from).unwrap_or_default());
        }
        Ok(CompletionType::Normal)
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let key = raw_context.vm.pop();
        let value = raw_context.vm.pop();
        let object = if let Some(object) = value.as_object() {
            object.clone()
        } else {
            value.to_object(context)?
        };

        let key = key.to_property_key(context)?;

        // Fast path:
        if object.is_array() {
            if let PropertyKey::Index(index) = &key {
                let object_borrowed = object.borrow();
                if let Some(element) = object_borrowed
                    .properties()
                    .dense_indexed_properties()
                    .and_then(|vec| vec.get(*index as usize))
                {
                    {
                        let context = context.as_raw_context_mut();
                        context.vm.push(key);
                        context.vm.push(element.clone());
                    }
                    return Ok(CompletionType::Normal);
                }
            }
        }

        // Slow path:
        let result = object.__get__(&key, value, context)?;

        {
            let context = context.as_raw_context_mut();
            context.vm.push(key);
            context.vm.push(result);
        }
        Ok(CompletionType::Normal)
    }
}
