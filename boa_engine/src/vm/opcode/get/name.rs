use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

/// `GetName` implements the Opcode Operation for `Opcode::GetName`
///
/// Operation:
///  - Find a binding on the environment chain and push its value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetName;

impl GetName {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let mut binding_locator = context.vm.frame().code_block.bindings[index];
        context.find_runtime_binding(&mut binding_locator)?;
        let value = context.get_binding(binding_locator)?.ok_or_else(|| {
            let name = context
                .interner()
                .resolve_expect(binding_locator.name().sym())
                .to_string();
            JsNativeError::reference().with_message(format!("{name} is not defined"))
        })?;

        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetName {
    const NAME: &'static str = "GetName";
    const INSTRUCTION: &'static str = "INST - GetName";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `GetGlobalName` implements the Opcode Operation for `Opcode::GetGlobalName`
///
/// Operation:
///  - TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetGlobalName;

impl GetGlobalName {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let key = context.vm.frame().code_block().names[index].clone();
        let global = context.global_object();
        if !global.has_property(key.clone(), context)? {
            return Err(JsNativeError::reference()
                .with_message(format!("{} is not defined", key.to_std_string_escaped()))
                .into());
        }

        let value = global.get(key, context)?;
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetGlobalName {
    const NAME: &'static str = "GetGlobalName";
    const INSTRUCTION: &'static str = "INST - GetGlobalName";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `GetGlobalNameOrUndefined` implements the Opcode Operation for `Opcode::GetGlobalNameOrUndefined`
///
/// Operation:
///  - TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetGlobalNameOrUndefined;

impl GetGlobalNameOrUndefined {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let key = context.vm.frame().code_block().names[index].clone();
        let global = context.global_object();
        if !global.has_property(key.clone(), context)? {
            context.vm.push(JsValue::undefined());
            return Ok(CompletionType::Normal);
        }

        let value = global.get(key, context)?;
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetGlobalNameOrUndefined {
    const NAME: &'static str = "GetGlobalNameOrUndefined";
    const INSTRUCTION: &'static str = "INST - GetGlobalNameOrUndefined";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `GetLocator` implements the Opcode Operation for `Opcode::GetLocator`
///
/// Operation:
///  - Find a binding on the environment and set the `current_binding` of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetLocator;

impl GetLocator {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let mut binding_locator = context.vm.frame().code_block.bindings[index];
        context.find_runtime_binding(&mut binding_locator)?;

        context.vm.frame_mut().binding_stack.push(binding_locator);

        Ok(CompletionType::Normal)
    }
}

impl Operation for GetLocator {
    const NAME: &'static str = "GetLocator";
    const INSTRUCTION: &'static str = "INST - GetLocator";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>();
        Self::operation(context, index as usize)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        Self::operation(context, index as usize)
    }
}

/// `GetNameAndLocator` implements the Opcode Operation for `Opcode::GetNameAndLocator`
///
/// Operation:
///  - Find a binding on the environment chain and push its value to the stack, setting the
/// `current_binding` of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetNameAndLocator;

impl GetNameAndLocator {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let mut binding_locator = context.vm.frame().code_block.bindings[index];
        context.find_runtime_binding(&mut binding_locator)?;
        let value = context.get_binding(binding_locator)?.ok_or_else(|| {
            let name = context
                .interner()
                .resolve_expect(binding_locator.name().sym())
                .to_string();
            JsNativeError::reference().with_message(format!("{name} is not defined"))
        })?;

        context.vm.frame_mut().binding_stack.push(binding_locator);
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetNameAndLocator {
    const NAME: &'static str = "GetNameAndLocator";
    const INSTRUCTION: &'static str = "INST - GetNameAndLocator";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>();
        Self::operation(context, index as usize)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        Self::operation(context, index as usize)
    }
}

/// `GetNameOrUndefined` implements the Opcode Operation for `Opcode::GetNameOrUndefined`
///
/// Operation:
///  - Find a binding on the environment chain and push its value. If the binding does not exist push undefined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetNameOrUndefined;

impl GetNameOrUndefined {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let mut binding_locator = context.vm.frame().code_block.bindings[index];

        let is_global = binding_locator.is_global();

        context.find_runtime_binding(&mut binding_locator)?;

        let value = if let Some(value) = context.get_binding(binding_locator)? {
            value
        } else if is_global {
            JsValue::undefined()
        } else {
            let name = context
                .interner()
                .resolve_expect(binding_locator.name().sym())
                .to_string();
            return Err(JsNativeError::reference()
                .with_message(format!("{name} is not defined"))
                .into());
        };

        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetNameOrUndefined {
    const NAME: &'static str = "GetNameOrUndefined";
    const INSTRUCTION: &'static str = "INST - GetNameOrUndefined";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>();
        Self::operation(context, index as usize)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        Self::operation(context, index as usize)
    }
}
