use boa_string::StaticJsStrings;

use crate::{
    Context, JsResult, JsValue, js_string,
    object::{internal_methods::InternalMethodPropertyContext, shape::slot::SlotAttributes},
    property::PropertyKey,
    vm::opcode::{Operation, VaryingOperand},
};

fn js_string_get(this: &JsValue, key: &PropertyKey) -> Option<JsValue> {
    let this = this.as_string()?;
    match key {
        PropertyKey::String(name) if *name == StaticJsStrings::LENGTH => Some(this.len().into()),
        PropertyKey::Index(index) => Some(
            this.get(index.get() as usize)
                .map_or_else(JsValue::undefined, |char| {
                    js_string!([char].as_slice()).into()
                }),
        ),
        _ => None,
    }
}

pub(crate) fn get_by_name<const LENGTH: bool>(
    (dst, object, receiver, index): (VaryingOperand, &JsValue, &JsValue, VaryingOperand),
    context: &mut Context,
) -> JsResult<()> {
    if LENGTH {
        if let Some(object) = object.as_object()
            && object.is_array()
        {
            let value = object.borrow().properties().storage[0].clone();
            context.vm.set_register(dst.into(), value);
            return Ok(());
        } else if let Some(string) = object.as_string() {
            context
                .vm
                .set_register(dst.into(), (string.len() as u32).into());
            return Ok(());
        }
    }

    let object = object.base_class(context)?;

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
            result =
                result
                    .as_object()
                    .expect("should contain getter")
                    .call(receiver, &[], context)?;
        }
        context.vm.set_register(dst.into(), result);
        return Ok(());
    }

    drop(object_borrowed);

    let key: PropertyKey = ic.name.clone().into();

    let context = &mut InternalMethodPropertyContext::new(context);
    let result = object.__get__(&key, receiver.clone(), context)?;

    // Cache the property.
    let slot = *context.slot();
    if slot.is_cachable() {
        let ic = &context.vm.frame().code_block.ic[usize::from(index)];
        let object_borrowed = object.borrow();
        let shape = object_borrowed.shape();
        ic.set(shape, slot);
    }

    context.vm.set_register(dst.into(), result);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GetLengthProperty;

impl GetLengthProperty {
    #[inline(always)]
    pub(crate) fn operation(
        (dst, object, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let object = context.vm.get_register(object.into()).clone();
        get_by_name::<true>((dst, &object, &object, index), context)
    }
}

impl Operation for GetLengthProperty {
    const NAME: &'static str = "GetLengthProperty";
    const INSTRUCTION: &'static str = "INST - GetLengthProperty";
    const COST: u8 = 4;
}

/// `GetPropertyByName` implements the Opcode Operation for `Opcode::GetPropertyByName`
///
/// Operation:
///  - Get a property by name from an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPropertyByName;

impl GetPropertyByName {
    #[inline(always)]
    pub(crate) fn operation(
        (dst, object, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let object = context.vm.get_register(object.into()).clone();
        get_by_name::<false>((dst, &object, &object, index), context)
    }
}

impl Operation for GetPropertyByName {
    const NAME: &'static str = "GetPropertyByName";
    const INSTRUCTION: &'static str = "INST - GetPropertyByName";
    const COST: u8 = 4;
}

/// `GetPropertyByNameWithThis` implements the Opcode Operation for `Opcode::GetPropertyByNameWithThis`
///
/// Operation:
///  - Get a property by name from an object with this.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPropertyByNameWithThis;

impl GetPropertyByNameWithThis {
    #[inline(always)]
    pub(crate) fn operation(
        (dst, receiver, value, index): (
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
        ),
        context: &mut Context,
    ) -> JsResult<()> {
        let receiver = context.vm.get_register(receiver.into()).clone();
        let object = context.vm.get_register(value.into()).clone();
        get_by_name::<false>((dst, &object, &receiver, index), context)
    }
}

impl Operation for GetPropertyByNameWithThis {
    const NAME: &'static str = "GetPropertyByNameWithThis";
    const INSTRUCTION: &'static str = "INST - GetPropertyByNameWithThis";
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
        context: &mut Context,
    ) -> JsResult<()> {
        let key = context.vm.get_register(key.into()).clone();
        let base = context.vm.get_register(object.into()).clone();
        let object = base.base_class(context)?;
        let key = key.to_property_key(context)?;

        // Fast Path
        if object.is_array()
            && let PropertyKey::Index(index) = &key
        {
            let object_borrowed = object.borrow();
            if let Some(element) = object_borrowed.properties().get_dense_property(index.get()) {
                context.vm.set_register(dst.into(), element);
                return Ok(());
            }
        } else if let Some(value) = js_string_get(&base, &key) {
            context.vm.set_register(dst.into(), value);
            return Ok(());
        }

        let receiver = context.vm.get_register(receiver.into());

        // Slow path:
        let result = object.__get__(
            &key,
            receiver.clone(),
            &mut InternalMethodPropertyContext::new(context),
        )?;

        context.vm.set_register(dst.into(), result);
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
        context: &mut Context,
    ) -> JsResult<()> {
        let key_value = context.vm.get_register(key.into()).clone();
        let base = context.vm.get_register(object.into()).clone();
        let object = base.base_class(context)?;
        let key_value = key_value.to_property_key(context)?;

        // Fast Path
        if object.is_array()
            && let PropertyKey::Index(index) = &key_value
        {
            let object_borrowed = object.borrow();
            if let Some(element) = object_borrowed.properties().get_dense_property(index.get()) {
                context.vm.set_register(key.into(), key_value.into());
                context.vm.set_register(dst.into(), element);
                return Ok(());
            }
        } else if let Some(value) = js_string_get(&base, &key_value) {
            context.vm.set_register(key.into(), key_value.into());
            context.vm.set_register(dst.into(), value);
            return Ok(());
        }

        let receiver = context.vm.get_register(receiver.into());

        // Slow path:
        let result = object.__get__(
            &key_value,
            receiver.clone(),
            &mut InternalMethodPropertyContext::new(context),
        )?;

        context.vm.set_register(key.into(), key_value.into());
        context.vm.set_register(dst.into(), result);
        Ok(())
    }
}

impl Operation for GetPropertyByValuePush {
    const NAME: &'static str = "GetPropertyByValuePush";
    const INSTRUCTION: &'static str = "INST - GetPropertyByValuePush";
    const COST: u8 = 4;
}
