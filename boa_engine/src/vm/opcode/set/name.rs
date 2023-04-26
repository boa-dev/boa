use crate::{
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

        if !context.is_initialized_binding(&binding_locator)? {
            if binding_locator.is_global() && context.vm.frame().code_block.strict {
                let key = context
                    .interner()
                    .resolve_expect(binding_locator.name().sym())
                    .to_string();

                return Err(JsNativeError::reference()
                    .with_message(format!(
                        "cannot assign to uninitialized global property `{key}`"
                    ))
                    .into());
            }

            if !binding_locator.is_global() {
                let key = context
                    .interner()
                    .resolve_expect(binding_locator.name().sym())
                    .to_string();

                return Err(JsNativeError::reference()
                    .with_message(format!("cannot assign to uninitialized binding `{key}`"))
                    .into());
            }
        }

        context.set_binding(binding_locator, value, context.vm.frame().code_block.strict)?;

        Ok(CompletionType::Normal)
    }
}
