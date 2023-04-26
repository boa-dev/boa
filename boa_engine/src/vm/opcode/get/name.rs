use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `GetName` implements the Opcode Operation for `Opcode::GetName`
///
/// Operation:
///  - Find a binding on the environment chain and push its value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetName;

impl Operation for GetName {
    const NAME: &'static str = "GetName";
    const INSTRUCTION: &'static str = "INST - GetName";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let mut binding_locator = context.vm.frame().code_block.bindings[index as usize];
        binding_locator.throw_mutate_immutable(context)?;
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

/// `GetNameAndLocator` implements the Opcode Operation for `Opcode::GetNameAndLocator`
///
/// Operation:
///  - Find a binding on the environment chain and push its value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetNameAndLocator;

impl Operation for GetNameAndLocator {
    const NAME: &'static str = "GetNameAndLocator";
    const INSTRUCTION: &'static str = "INST - GetNameAndLocator";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let mut binding_locator = context.vm.frame().code_block.bindings[index as usize];
        binding_locator.throw_mutate_immutable(context)?;
        context.find_runtime_binding(&mut binding_locator)?;
        let value = context.get_binding(binding_locator)?.ok_or_else(|| {
            let name = context
                .interner()
                .resolve_expect(binding_locator.name().sym())
                .to_string();
            JsNativeError::reference().with_message(format!("{name} is not defined"))
        })?;

        context.vm.bindings_stack.push(binding_locator);
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

/// `GetNameOrUndefined` implements the Opcode Operation for `Opcode::GetNameOrUndefined`
///
/// Operation:
///  - Find a binding on the environment chain and push its value. If the binding does not exist push undefined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetNameOrUndefined;

impl Operation for GetNameOrUndefined {
    const NAME: &'static str = "GetNameOrUndefined";
    const INSTRUCTION: &'static str = "INST - GetNameOrUndefined";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let mut binding_locator = context.vm.frame().code_block.bindings[index as usize];
        binding_locator.throw_mutate_immutable(context)?;

        context.find_runtime_binding(&mut binding_locator)?;

        let value = context.get_binding(binding_locator)?.unwrap_or_default();

        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}
