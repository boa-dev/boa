use crate::{
    object::{internal_methods::InternalMethodContext, shape::slot::SlotAttributes},
    property::PropertyKey,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `GetPropertyByName` implements the Opcode Operation for `Opcode::GetPropertyByName`
///
/// Operation:
///  - Get a property by name from an object an push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPropertyByName;

impl GetPropertyByName {
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let receiver = context.vm.pop();
        let value = context.vm.pop();
        let object = if let Some(object) = value.as_object() {
            object.clone()
        } else {
            value.to_object(context)?
        };

        let ic = &context.vm.frame().code_block().ic[index];
        let mut slot = ic.slot();
        if slot.is_cachable() {
            let object_borrowed = object.borrow();
            if ic.matches(object_borrowed.shape()) {
                let mut result = if slot.attributes.contains(SlotAttributes::PROTOTYPE) {
                    let prototype = object
                        .borrow()
                        .properties()
                        .shape
                        .prototype()
                        .expect("prototype should have value");
                    let prototype = prototype.borrow();
                    prototype.properties().storage[slot.index as usize].clone()
                } else {
                    object_borrowed.properties().storage[slot.index as usize].clone()
                };

                drop(object_borrowed);
                if slot.attributes.has_get() && result.is_object() {
                    result = result.as_object().expect("should contain getter").call(
                        &receiver,
                        &[],
                        context,
                    )?;
                }
                context.vm.push(result);
                return Ok(CompletionType::Normal);
            }
        }

        let key: PropertyKey = ic.name.clone().into();

        let context = &mut InternalMethodContext::new(context);
        let result = object.__get__(&key, receiver, context)?;

        slot = *context.slot();

        // Cache the property.
        if slot.attributes.is_cachable() {
            let ic = &context.vm.frame().code_block.ic[index];
            let object_borrowed = object.borrow();
            let shape = object_borrowed.shape();
            ic.set(shape, slot);
        }

        context.vm.push(result);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetPropertyByName {
    const NAME: &'static str = "GetPropertyByName";
    const INSTRUCTION: &'static str = "INST - GetPropertyByName";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>();
        Self::operation(context, index as usize)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        Self::operation(context, index as usize)
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
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let key = context.vm.pop();
        let receiver = context.vm.pop();
        let value = context.vm.pop();
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
                    .and_then(|vec| vec.get(index.get() as usize))
                {
                    context.vm.push(element.clone());
                    return Ok(CompletionType::Normal);
                }
            }
        }

        // Slow path:
        let result = object.__get__(&key, receiver, &mut InternalMethodContext::new(context))?;

        context.vm.push(result);
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
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let key = context.vm.pop();
        let receiver = context.vm.pop();
        let value = context.vm.pop();
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
                    .and_then(|vec| vec.get(index.get() as usize))
                {
                    context.vm.push(key);
                    context.vm.push(element.clone());
                    return Ok(CompletionType::Normal);
                }
            }
        }

        // Slow path:
        let result = object.__get__(&key, receiver, &mut InternalMethodContext::new(context))?;

        context.vm.push(key);
        context.vm.push(result);
        Ok(CompletionType::Normal)
    }
}
