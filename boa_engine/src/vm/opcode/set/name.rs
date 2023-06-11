use crate::{
    environments::{BindingLocator, Environment},
    vm::{opcode::Operation, CompletionType},
    Context, JsNativeError, JsResult,
};

/// `ThrowMutateImmutable` implements the Opcode Operation for `Opcode::ThrowMutateImmutable`
///
/// Operation:
///  - Throws an error because the binding access is illegal.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowMutateImmutable;

impl Operation for ThrowMutateImmutable {
    const NAME: &'static str = "ThrowMutateImmutable";
    const INSTRUCTION: &'static str = "INST - ThrowMutateImmutable";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let name = &context.vm.frame().code_block.names[index as usize];

        Err(JsNativeError::typ()
            .with_message(format!(
                "cannot mutate an immutable binding '{}'",
                name.to_std_string_escaped()
            ))
            .into())
    }
}

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

        context.find_runtime_binding(&mut binding_locator)?;

        verify_initialized(binding_locator, context)?;

        context.set_binding(
            binding_locator,
            value,
            context.vm.frame().code_block.strict(),
        )?;

        Ok(CompletionType::Normal)
    }
}

/// `SetNameByLocator` implements the Opcode Operation for `Opcode::SetNameByLocator`
///
/// Operation:
///  - Assigns a value to the binding pointed by the `current_binding` of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetNameByLocator;

impl Operation for SetNameByLocator {
    const NAME: &'static str = "SetNameByLocator";
    const INSTRUCTION: &'static str = "INST - SetNameByLocator";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let binding_locator = context
            .vm
            .frame_mut()
            .binding_stack
            .pop()
            .expect("locator should have been popped before");
        let value = context.vm.pop();
        if binding_locator.is_silent() {
            return Ok(CompletionType::Normal);
        }

        verify_initialized(binding_locator, context)?;

        context.set_binding(
            binding_locator,
            value,
            context.vm.frame().code_block.strict(),
        )?;

        Ok(CompletionType::Normal)
    }
}

/// Checks that the binding pointed by `locator` exists and is initialized.
fn verify_initialized(locator: BindingLocator, context: &mut Context<'_>) -> JsResult<()> {
    if !context.is_initialized_binding(&locator)? {
        let key = context.interner().resolve_expect(locator.name().sym());
        let strict = context.vm.frame().code_block.strict();

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
