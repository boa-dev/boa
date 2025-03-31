use crate::{
    object::{internal_methods::InternalMethodContext, shape::slot::SlotAttributes},
    property::PropertyKey,
    vm::{
        opcode::{Operation, VaryingOperand},
        Registers,
    },
    Context, JsResult,
};

/// `GetPropertyByName` implements the Opcode Operation for `Opcode::GetPropertyByName`
///
/// Operation:
///  - Get a property by name from an object an push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPropertyByName;

impl GetPropertyByName {
    #[inline(always)]
    pub(crate) fn operation(
        (dst, receiver, value, index): (
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
        ),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let receiver = registers.get(receiver.into());
        let object = registers.get(value.into());
        let object = object.to_object(context)?;

        let ic = &context.vm.frame().code_block().ic[usize::from(index)];
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
                    receiver,
                    &[],
                    context,
                )?;
            }
            registers.set(dst.into(), result);
            return Ok(());
        }

        drop(object_borrowed);

        let key: PropertyKey = ic.name.clone().into();

        let context = &mut InternalMethodContext::new(context);
        let result = object.__get__(&key, receiver.clone(), context)?;

        // Cache the property.
        let slot = *context.slot();
        if slot.is_cachable() {
            let ic = &context.vm.frame().code_block.ic[usize::from(index)];
            let object_borrowed = object.borrow();
            let shape = object_borrowed.shape();
            ic.set(shape, slot);
        }

        registers.set(dst.into(), result);
        Ok(())
    }
}

impl Operation for GetPropertyByName {
    const NAME: &'static str = "GetPropertyByName";
    const INSTRUCTION: &'static str = "INST - GetPropertyByName";
    const COST: u8 = 4;
}

/// `GetPropertyByValue` implements the Opcode Operation for `Opcode::GetPropertyByValue`
///
/// Operation:
///  - Get a property by value from an object an push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPropertyByValue;

impl GetPropertyByValue {
    #[inline(always)]
    pub(crate) fn operation(
        (dst, key, receiver, object): (
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
        ),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let key = registers.get(key.into());
        let object = registers.get(object.into());
        let object = object.to_object(context)?;
        let key = key.to_property_key(context)?;

        // Fast Path
        if object.is_array() {
            if let PropertyKey::Index(index) = &key {
                let object_borrowed = object.borrow();
                if let Some(element) = object_borrowed.properties().get_dense_property(index.get())
                {
                    registers.set(dst.into(), element);
                    return Ok(());
                }
            }
        }

        let receiver = registers.get(receiver.into());

        // Slow path:
        let result = object.__get__(
            &key,
            receiver.clone(),
            &mut InternalMethodContext::new(context),
        )?;

        registers.set(dst.into(), result);
        Ok(())
    }
}

impl Operation for GetPropertyByValue {
    const NAME: &'static str = "GetPropertyByValue";
    const INSTRUCTION: &'static str = "INST - GetPropertyByValue";
    const COST: u8 = 4;
}

/// `GetPropertyByValuePush` implements the Opcode Operation for `Opcode::GetPropertyByValuePush`
///
/// Operation:
///  - Get a property by value from an object an push the key and value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPropertyByValuePush;

impl GetPropertyByValuePush {
    #[inline(always)]
    pub(crate) fn operation(
        (dst, key, receiver, object): (
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
        ),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let key_value = registers.get(key.into());
        let object = registers.get(object.into());
        let object = object.to_object(context)?;
        let key_value = key_value.to_property_key(context)?;

        // Fast Path
        if object.is_array() {
            if let PropertyKey::Index(index) = &key_value {
                let object_borrowed = object.borrow();
                if let Some(element) = object_borrowed.properties().get_dense_property(index.get())
                {
                    registers.set(key.into(), key_value.into());
                    registers.set(dst.into(), element);
                    return Ok(());
                }
            }
        }

        let receiver = registers.get(receiver.into());

        // Slow path:
        let result = object.__get__(
            &key_value,
            receiver.clone(),
            &mut InternalMethodContext::new(context),
        )?;

        registers.set(key.into(), key_value.into());
        registers.set(dst.into(), result);
        Ok(())
    }
}

impl Operation for GetPropertyByValuePush {
    const NAME: &'static str = "GetPropertyByValuePush";
    const INSTRUCTION: &'static str = "INST - GetPropertyByValuePush";
    const COST: u8 = 4;
}
