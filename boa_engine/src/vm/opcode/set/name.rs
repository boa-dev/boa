use crate::{
    environments::{BindingLocator, Environment},
    vm::{opcode::Operation, CompletionType},
    Context, JsNativeError, JsResult,
};

/// `SetName` implements the Opcode Operation for `Opcode::SetName`
///
/// Operation:
///  - Find a binding on the environment chain and assign its value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetName;

impl Operation for SetName {
    const NAME: &'static str = "SetName";
    const INSTRUCTION: &'static str = "INST - SetName";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let mut binding_locator = context.vm.frame().code_block.bindings[index as usize];
        let value = context.vm.pop();
        if binding_locator.is_silent() {
            return Ok(CompletionType::Normal);
        }
        binding_locator.throw_mutate_immutable(context)?;

        context.find_runtime_binding(&mut binding_locator)?;

        verify_initialized(binding_locator, context)?;

        context.set_binding(binding_locator, value, context.vm.frame().code_block.strict)?;

        Ok(CompletionType::Normal)
    }
}

/// `SetNameByBinding` implements the Opcode Operation for `Opcode::SetNameByBinding`
///
/// Operation:
///  - Assigns a value to the binding pointed by the `current_binding` register.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetNameByBinding;

impl Operation for SetNameByBinding {
    const NAME: &'static str = "SetNameByBinding";
    const INSTRUCTION: &'static str = "INST - SetNameByBinding";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let binding_locator = context
            .vm
            .bindings_stack
            .pop()
            .expect("a binding should have been pushed before");
        let value = context.vm.pop();
        if binding_locator.is_silent() {
            return Ok(CompletionType::Normal);
        }
        binding_locator.throw_mutate_immutable(context)?;

        verify_initialized(binding_locator, context)?;

        context.set_binding(binding_locator, value, context.vm.frame().code_block.strict)?;

        Ok(CompletionType::Normal)
    }
}

/// Checks that the binding pointed by `locator` exists and is initialized.
fn verify_initialized(locator: BindingLocator, context: &mut Context<'_>) -> JsResult<()> {
    if !context.is_initialized_binding(&locator)? {
        let key = context.interner().resolve_expect(locator.name().sym());
        let strict = context.vm.frame().code_block.strict;

        let message = if locator.is_global() {
            strict.then(|| format!("cannot assign to uninitialized global property `{key}`"))
        } else {
            match context.environment_expect(locator.environment_index()) {
                Environment::Declarative(_) => {
                    Some(format!("cannot assign to uninitialized binding `{key}`"))
                }
                Environment::Object(_) if strict => {
                    Some(format!("cannot assign to uninitialized property `{key}`"))
                }
                Environment::Object(_) => None,
            }
        };

        if let Some(message) = message {
            return Err(JsNativeError::reference().with_message(message).into());
        }
    }

    Ok(())
}
