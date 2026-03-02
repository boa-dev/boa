use boa_ast::scope::{BindingLocator, BindingLocatorScope};

use crate::{
    Context, JsError, JsNativeError, JsResult,
    environments::Environment,
    vm::opcode::{Operation, VaryingOperand},
};

/// `ThrowMutateImmutable` implements the Opcode Operation for `Opcode::ThrowMutateImmutable`
///
/// Operation:
///  - Throws an error because the binding access is illegal.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowMutateImmutable;

impl ThrowMutateImmutable {
    #[inline(always)]
    pub(crate) fn operation(index: VaryingOperand, context: &Context) -> JsError {
        let name = context.with_vm(|vm| vm.frame().code_block().constant_string(index.into()));

        JsNativeError::typ()
            .with_message(format!(
                "cannot mutate an immutable binding '{}'",
                name.to_std_string_escaped()
            ))
            .into()
    }
}

impl Operation for ThrowMutateImmutable {
    const NAME: &'static str = "ThrowMutateImmutable";
    const INSTRUCTION: &'static str = "INST - ThrowMutateImmutable";
    const COST: u8 = 2;
}

/// `SetName` implements the Opcode Operation for `Opcode::SetName`
///
/// Operation:
///  - Find a binding on the environment chain and assign its value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetName;

impl SetName {
    #[inline(always)]
    pub(crate) fn operation(
        (value, index): (VaryingOperand, VaryingOperand),
        context: &Context,
    ) -> JsResult<()> {
        let value = context.get_register(value.into()).clone();
        let (mut binding_locator, strict) = context.with_vm(|vm| {
            let code_block = vm.frame().code_block();
            (
                code_block.bindings[usize::from(index)].clone(),
                code_block.strict(),
            )
        });

        context.find_runtime_binding(&mut binding_locator)?;

        verify_initialized(&binding_locator, context)?;

        context.set_binding(&binding_locator, value, strict)?;

        Ok(())
    }
}

impl Operation for SetName {
    const NAME: &'static str = "SetName";
    const INSTRUCTION: &'static str = "INST - SetName";
    const COST: u8 = 4;
}

/// `SetNameByLocator` implements the Opcode Operation for `Opcode::SetNameByLocator`
///
/// Operation:
///  - Assigns a value to the binding pointed by the `current_binding` of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetNameByLocator;

impl SetNameByLocator {
    #[inline(always)]
    pub(crate) fn operation(value: VaryingOperand, context: &Context) -> JsResult<()> {
        let (strict, binding_locator) = context.with_vm_mut(|vm| {
            let frame = vm.frame_mut();
            let strict = frame.code_block.strict();
            let binding_locator = frame
                .binding_stack
                .pop()
                .expect("locator should have been popped before");
            (strict, binding_locator)
        });
        let value = context.get_register(value.into()).clone();

        verify_initialized(&binding_locator, context)?;

        context.set_binding(&binding_locator, value, strict)?;

        Ok(())
    }
}

impl Operation for SetNameByLocator {
    const NAME: &'static str = "SetNameByLocator";
    const INSTRUCTION: &'static str = "INST - SetNameByLocator";
    const COST: u8 = 4;
}

/// Checks that the binding pointed by `locator` exists and is initialized.
fn verify_initialized(locator: &BindingLocator, context: &Context) -> JsResult<()> {
    if !context.is_initialized_binding(locator)? {
        let key = locator.name();
        let strict = context.with_vm(|vm| vm.frame().code_block.strict());

        let message = match locator.scope() {
            BindingLocatorScope::GlobalObject if strict => Some(format!(
                "cannot assign to uninitialized global property `{}`",
                key.to_std_string_escaped()
            )),
            BindingLocatorScope::GlobalObject => None,
            BindingLocatorScope::GlobalDeclarative => Some(format!(
                "cannot assign to uninitialized binding `{}`",
                key.to_std_string_escaped()
            )),
            BindingLocatorScope::Stack(index) => match &context.environment_expect(index) {
                Environment::Declarative(_) => Some(format!(
                    "cannot assign to uninitialized binding `{}`",
                    key.to_std_string_escaped()
                )),
                Environment::Object(_) if strict => Some(format!(
                    "cannot assign to uninitialized property `{}`",
                    key.to_std_string_escaped()
                )),
                Environment::Object(_) => None,
            },
        };

        if let Some(message) = message {
            return Err(JsNativeError::reference().with_message(message).into());
        }
    }

    Ok(())
}
