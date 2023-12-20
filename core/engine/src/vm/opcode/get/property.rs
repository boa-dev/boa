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
    fn operation(
        dst: u32,
        receiver: u32,
        value: u32,
        index: usize,
        operand_types: u8,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let receiver = context
            .vm
            .frame()
            .read_value::<0>(operand_types, receiver, &context.vm);
        let value = context
            .vm
            .frame()
            .read_value::<1>(operand_types, value, &context.vm);

        let rp = context.vm.frame().rp;
        let object = if let Some(object) = value.as_object() {
            object.clone()
        } else {
            value.clone().to_object(context)?
        };

        let ic = &context.vm.frame().code_block().ic[index];
        let object_borrowed = object.borrow();
        if let Some((shape, slot)) = ic.match_or_reset(object_borrowed.shape()) {
            let mut result = if slot.attributes.contains(SlotAttributes::PROTOTYPE) {
                let prototype = shape.prototype().expect("prototype should have value");
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
            context.vm.stack[(rp + dst) as usize] = result;
            return Ok(CompletionType::Normal);
        }

        drop(object_borrowed);

        let key: PropertyKey = ic.name.clone().into();

        let context = &mut InternalMethodContext::new(context);
        let result = object.__get__(&key, receiver, context)?;

        // Cache the property.
        let slot = *context.slot();
        if slot.is_cachable() {
            let ic = &context.vm.frame().code_block.ic[index];
            let object_borrowed = object.borrow();
            let shape = object_borrowed.shape();
            ic.set(shape, slot);
        }

        context.vm.stack[(rp + dst) as usize] = result;
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetPropertyByName {
    const NAME: &'static str = "GetPropertyByName";
    const INSTRUCTION: &'static str = "INST - GetPropertyByName";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u8>().into();
        let receiver = context.vm.read::<u8>().into();
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(dst, receiver, value, index, operand_types, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u16>().into();
        let receiver = context.vm.read::<u16>().into();
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(dst, receiver, value, index, operand_types, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u32>();
        let receiver = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(dst, receiver, value, index, operand_types, context)
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
                if let Some(element) = object_borrowed.properties().get_dense_property(index.get())
                {
                    context.vm.push(element);
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
                if let Some(element) = object_borrowed.properties().get_dense_property(index.get())
                {
                    context.vm.push(key);
                    context.vm.push(element);
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
