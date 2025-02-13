use crate::{
    error::JsNativeError,
    object::{internal_methods::InternalMethodContext, shape::slot::SlotAttributes},
    property::PropertyKey,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult, JsValue,
};

/// `GetName` implements the Opcode Operation for `Opcode::GetName`
///
/// Operation:
///  - Find a binding on the environment chain and push its value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetName;

impl GetName {
    fn operation(
        value: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let mut binding_locator = context.vm.frame().code_block.bindings[index].clone();
        context.find_runtime_binding(&mut binding_locator)?;
        let result = context.get_binding(&binding_locator)?.ok_or_else(|| {
            let name = binding_locator.name().to_std_string_escaped();
            JsNativeError::reference().with_message(format!("{name} is not defined"))
        })?;
        registers.set(value, result);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetName {
    const NAME: &'static str = "GetName";
    const INSTRUCTION: &'static str = "INST - GetName";
    const COST: u8 = 4;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(value, index, registers, context)
    }
}

/// `GetNameGlobal` implements the Opcode Operation for `Opcode::GetNameGlobal`
///
/// Operation:
///  - Find a binding in the global object and push its value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetNameGlobal;

impl GetNameGlobal {
    fn operation(
        dst: u32,
        index: usize,
        ic_index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let mut binding_locator = context.vm.frame().code_block.bindings[index].clone();
        context.find_runtime_binding(&mut binding_locator)?;

        if binding_locator.is_global() {
            let object = context.global_object();

            let ic = &context.vm.frame().code_block().ic[ic_index];

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
                        &object.clone().into(),
                        &[],
                        context,
                    )?;
                }
                registers.set(dst, result);
                return Ok(CompletionType::Normal);
            }

            drop(object_borrowed);

            let key: PropertyKey = ic.name.clone().into();

            let context = &mut InternalMethodContext::new(context);
            let Some(result) = object.__try_get__(&key, object.clone().into(), context)? else {
                let name = binding_locator.name().to_std_string_escaped();
                return Err(JsNativeError::reference()
                    .with_message(format!("{name} is not defined"))
                    .into());
            };

            // Cache the property.
            let slot = *context.slot();
            if slot.is_cachable() {
                let ic = &context.vm.frame().code_block.ic[ic_index];
                let object_borrowed = object.borrow();
                let shape = object_borrowed.shape();
                ic.set(shape, slot);
            }

            registers.set(dst, result);
            return Ok(CompletionType::Normal);
        }

        let result = context.get_binding(&binding_locator)?.ok_or_else(|| {
            let name = binding_locator.name().to_std_string_escaped();
            JsNativeError::reference().with_message(format!("{name} is not defined"))
        })?;

        registers.set(dst, result);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetNameGlobal {
    const NAME: &'static str = "GetNameGlobal";
    const INSTRUCTION: &'static str = "INST - GetNameGlobal";
    const COST: u8 = 4;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        let ic_index = context.vm.read::<u8>() as usize;
        Self::operation(dst, index, ic_index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        let ic_index = context.vm.read::<u16>() as usize;
        Self::operation(dst, index, ic_index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        let ic_index = context.vm.read::<u32>() as usize;
        Self::operation(dst, index, ic_index, registers, context)
    }
}

/// `GetLocator` implements the Opcode Operation for `Opcode::GetLocator`
///
/// Operation:
///  - Find a binding on the environment and set the `current_binding` of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetLocator;

impl GetLocator {
    fn operation(index: usize, context: &mut Context) -> JsResult<CompletionType> {
        let mut binding_locator = context.vm.frame().code_block.bindings[index].clone();
        context.find_runtime_binding(&mut binding_locator)?;

        context.vm.frame_mut().binding_stack.push(binding_locator);

        Ok(CompletionType::Normal)
    }
}

impl Operation for GetLocator {
    const NAME: &'static str = "GetLocator";
    const INSTRUCTION: &'static str = "INST - GetLocator";
    const COST: u8 = 4;

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(index, context)
    }

    fn execute_u16(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(index, context)
    }

    fn execute_u32(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(index, context)
    }
}

/// `GetNameAndLocator` implements the Opcode Operation for `Opcode::GetNameAndLocator`
///
/// Operation:
///  - Find a binding on the environment chain and push its value to the stack, setting the
///    `current_binding` of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetNameAndLocator;

impl GetNameAndLocator {
    fn operation(
        value: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let mut binding_locator = context.vm.frame().code_block.bindings[index].clone();
        context.find_runtime_binding(&mut binding_locator)?;
        let result = context.get_binding(&binding_locator)?.ok_or_else(|| {
            let name = binding_locator.name().to_std_string_escaped();
            JsNativeError::reference().with_message(format!("{name} is not defined"))
        })?;

        context.vm.frame_mut().binding_stack.push(binding_locator);
        registers.set(value, result);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetNameAndLocator {
    const NAME: &'static str = "GetNameAndLocator";
    const INSTRUCTION: &'static str = "INST - GetNameAndLocator";
    const COST: u8 = 4;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(value, index, registers, context)
    }
}

/// `GetNameOrUndefined` implements the Opcode Operation for `Opcode::GetNameOrUndefined`
///
/// Operation:
///  - Find a binding on the environment chain and push its value. If the binding does not exist push undefined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetNameOrUndefined;

impl GetNameOrUndefined {
    fn operation(
        value: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let mut binding_locator = context.vm.frame().code_block.bindings[index].clone();

        let is_global = binding_locator.is_global();

        context.find_runtime_binding(&mut binding_locator)?;

        let result = if let Some(value) = context.get_binding(&binding_locator)? {
            value
        } else if is_global {
            JsValue::undefined()
        } else {
            let name = binding_locator.name().to_std_string_escaped();
            return Err(JsNativeError::reference()
                .with_message(format!("{name} is not defined"))
                .into());
        };

        registers.set(value, result);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetNameOrUndefined {
    const NAME: &'static str = "GetNameOrUndefined";
    const INSTRUCTION: &'static str = "INST - GetNameOrUndefined";
    const COST: u8 = 4;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(value, index, registers, context)
    }
}
