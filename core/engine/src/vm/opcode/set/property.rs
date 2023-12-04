use boa_macros::utf16;

use crate::{
    builtins::{function::set_function_name, Proxy},
    object::{internal_methods::InternalMethodContext, shape::slot::SlotAttributes},
    property::{PropertyDescriptor, PropertyKey},
    vm::{opcode::Operation, CompletionType},
    Context, JsNativeError, JsResult, JsString, JsValue,
};

/// `SetPropertyByName` implements the Opcode Operation for `Opcode::SetPropertyByName`
///
/// Operation:
///  - Sets a property by name of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertyByName;

impl SetPropertyByName {
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        let receiver = context.vm.pop();
        let object = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            object.to_object(context)?
        };

        let ic = &context.vm.frame().code_block().ic[index];

        let object_borrowed = object.borrow();
        if let Some((shape, slot)) = ic.match_or_reset(object_borrowed.shape()) {
            let slot_index = slot.index as usize;

            if slot.attributes.is_accessor_descriptor() {
                let result = if slot.attributes.contains(SlotAttributes::PROTOTYPE) {
                    let prototype = shape.prototype().expect("prototype should have value");
                    let prototype = prototype.borrow();

                    prototype.properties().storage[slot_index + 1].clone()
                } else {
                    object_borrowed.properties().storage[slot_index + 1].clone()
                };

                drop(object_borrowed);
                if slot.attributes.has_set() && result.is_object() {
                    result.as_object().expect("should contain getter").call(
                        &receiver,
                        &[value.clone()],
                        context,
                    )?;
                }
            } else if slot.attributes.contains(SlotAttributes::PROTOTYPE) {
                let prototype = shape.prototype().expect("prototype should have value");
                let mut prototype = prototype.borrow_mut();

                prototype.properties_mut().storage[slot_index] = value.clone();
            } else {
                drop(object_borrowed);
                let mut object_borrowed = object.borrow_mut();
                object_borrowed.properties_mut().storage[slot_index] = value.clone();
            }
            context.vm.push(value);
            return Ok(CompletionType::Normal);
        }
        drop(object_borrowed);

        let name: PropertyKey = ic.name.clone().into();

        let context = &mut InternalMethodContext::new(context);
        let succeeded = object.__set__(name.clone(), value.clone(), receiver, context)?;
        if !succeeded && context.vm.frame().code_block.strict() {
            return Err(JsNativeError::typ()
                .with_message(format!("cannot set non-writable property: {name}"))
                .into());
        }

        // Cache the property.
        let slot = *context.slot();
        if succeeded && slot.is_cachable() {
            let ic = &context.vm.frame().code_block.ic[index];
            let object_borrowed = object.borrow();
            let shape = object_borrowed.shape();
            ic.set(shape, slot);
        }
        context.vm.stack.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for SetPropertyByName {
    const NAME: &'static str = "SetPropertyByName";
    const INSTRUCTION: &'static str = "INST - SetPropertyByName";
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

/// `SetPropertyByValue` implements the Opcode Operation for `Opcode::SetPropertyByValue`
///
/// Operation:
///  - Sets a property by value of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertyByValue;

impl Operation for SetPropertyByValue {
    const NAME: &'static str = "SetPropertyByValue";
    const INSTRUCTION: &'static str = "INST - SetPropertyByValue";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let receiver = context.vm.pop();
        let object = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            object.to_object(context)?
        };

        let key = key.to_property_key(context)?;

        // Fast Path:
        'fast_path: {
            if object.is_array() {
                if let PropertyKey::Index(index) = &key {
                    let mut object_borrowed = object.borrow_mut();

                    // Cannot modify if not extensible.
                    if !object_borrowed.extensible {
                        break 'fast_path;
                    }

                    let shape = object_borrowed.shape().clone();

                    if let Some(dense_elements) = object_borrowed
                        .properties_mut()
                        .dense_indexed_properties_mut()
                    {
                        let index = index.get() as usize;
                        if let Some(element) = dense_elements.get_mut(index) {
                            *element = value;
                            context.vm.push(element.clone());
                            return Ok(CompletionType::Normal);
                        } else if dense_elements.len() == index {
                            // Cannot use fast path if the [[prototype]] is a proxy object,
                            // because we have to the call prototypes [[set]] on non-existing property,
                            // and proxy objects can override [[set]].
                            let prototype = shape.prototype();
                            if prototype.map_or(false, |x| x.is::<Proxy>()) {
                                break 'fast_path;
                            }

                            dense_elements.push(value.clone());
                            context.vm.push(value);

                            let len = dense_elements.len() as u32;
                            let length_key = PropertyKey::from(utf16!("length"));
                            let length = object_borrowed
                                .properties_mut()
                                .get(&length_key)
                                .expect("Arrays must have length property");

                            if length.expect_writable() {
                                // We have to get the max of previous length and len(dense_elements) + 1,
                                // this is needed if user spacifies `new Array(n)` then adds properties from 0, 1, etc.
                                let len = length
                                    .expect_value()
                                    .to_u32(context)
                                    .expect("length should have a u32 value")
                                    .max(len);
                                object_borrowed.insert(
                                    length_key,
                                    PropertyDescriptor::builder()
                                        .value(len)
                                        .writable(true)
                                        .enumerable(length.expect_enumerable())
                                        .configurable(false)
                                        .build(),
                                );
                            } else if context.vm.frame().code_block.strict() {
                                return Err(JsNativeError::typ().with_message("TypeError: Cannot assign to read only property 'length' of array object").into());
                            }
                            return Ok(CompletionType::Normal);
                        }
                    }
                }
            }
        }

        // Slow path:
        let succeeded =
            object.__set__(key.clone(), value.clone(), receiver, &mut context.into())?;
        if !succeeded && context.vm.frame().code_block.strict() {
            return Err(JsNativeError::typ()
                .with_message(format!("cannot set non-writable property: {key}"))
                .into());
        }
        context.vm.stack.push(value);
        Ok(CompletionType::Normal)
    }
}

/// `SetPropertyGetterByName` implements the Opcode Operation for `Opcode::SetPropertyGetterByName`
///
/// Operation:
///  - Sets a getter property by name of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertyGetterByName;

impl SetPropertyGetterByName {
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        let object = context.vm.pop();
        let object = object.to_object(context)?;
        let name = context
            .vm
            .frame()
            .code_block()
            .constant_string(index)
            .into();
        let set = object
            .__get_own_property__(&name, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        object.__define_own_property__(
            &name,
            PropertyDescriptor::builder()
                .maybe_get(Some(value))
                .maybe_set(set)
                .enumerable(true)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for SetPropertyGetterByName {
    const NAME: &'static str = "SetPropertyGetterByName";
    const INSTRUCTION: &'static str = "INST - SetPropertyGetterByName";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `SetPropertyGetterByValue` implements the Opcode Operation for `Opcode::SetPropertyGetterByValue`
///
/// Operation:
///  - Sets a getter property by value of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertyGetterByValue;

impl Operation for SetPropertyGetterByValue {
    const NAME: &'static str = "SetPropertyGetterByValue";
    const INSTRUCTION: &'static str = "INST - SetPropertyGetterByValue";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = object.to_object(context)?;
        let name = key.to_property_key(context)?;

        let set = object
            .__get_own_property__(&name, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        object.__define_own_property__(
            &name,
            PropertyDescriptor::builder()
                .maybe_get(Some(value))
                .maybe_set(set)
                .enumerable(true)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

/// `SetPropertySetterByName` implements the Opcode Operation for `Opcode::SetPropertySetterByName`
///
/// Operation:
///  - Sets a setter property by name of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertySetterByName;

impl SetPropertySetterByName {
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        let object = context.vm.pop();
        let object = object.to_object(context)?;
        let name = context
            .vm
            .frame()
            .code_block()
            .constant_string(index)
            .into();

        let get = object
            .__get_own_property__(&name, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();
        object.__define_own_property__(
            &name,
            PropertyDescriptor::builder()
                .maybe_set(Some(value))
                .maybe_get(get)
                .enumerable(true)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for SetPropertySetterByName {
    const NAME: &'static str = "SetPropertySetterByName";
    const INSTRUCTION: &'static str = "INST - SetPropertySetterByName";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `SetPropertySetterByValue` implements the Opcode Operation for `Opcode::SetPropertySetterByValue`
///
/// Operation:
///  - Sets a setter property by value of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPropertySetterByValue;

impl Operation for SetPropertySetterByValue {
    const NAME: &'static str = "SetPropertySetterByValue";
    const INSTRUCTION: &'static str = "INST - SetPropertySetterByValue";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = object.to_object(context)?;
        let name = key.to_property_key(context)?;

        let get = object
            .__get_own_property__(&name, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();
        object.__define_own_property__(
            &name,
            PropertyDescriptor::builder()
                .maybe_set(Some(value))
                .maybe_get(get)
                .enumerable(true)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

/// `SetFunctionName` implements the Opcode Operation for `Opcode::SetFunctionName`
///
/// Operation:
///  - Sets the name of a function object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetFunctionName;

impl Operation for SetFunctionName {
    const NAME: &'static str = "SetFunctionName";
    const INSTRUCTION: &'static str = "INST - SetFunctionName";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let prefix = context.vm.read::<u8>();
        let function = context.vm.pop();
        let name = context.vm.pop();

        let name = match name {
            JsValue::String(name) => name.into(),
            JsValue::Symbol(name) => name.into(),
            _ => unreachable!(),
        };

        let prefix = match prefix {
            1 => Some(JsString::from("get")),
            2 => Some(JsString::from("set")),
            _ => None,
        };

        set_function_name(
            function.as_object().expect("function is not an object"),
            &name,
            prefix,
            context,
        );

        context.vm.stack.push(function);
        Ok(CompletionType::Normal)
    }
}
