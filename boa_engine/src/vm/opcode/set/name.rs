use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsString,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SetName;

impl Operation for SetName {
    const NAME: &'static str = "SetName";
    const INSTRUCTION: &'static str = "INST - SetName";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let binding_locator = context.vm.frame().code.bindings[index as usize];
        let value = context.vm.pop();
        binding_locator.throw_mutate_immutable(context)?;

        if binding_locator.is_global() {
            if !context
                .realm
                .environments
                .put_value_global_poisoned(binding_locator.name(), &value)
            {
                let key: JsString = context
                    .interner()
                    .resolve_expect(binding_locator.name())
                    .into();
                let exists = context.global_bindings_mut().contains_key(&key);

                if !exists && context.vm.frame().code.strict {
                    return context
                        .throw_reference_error(format!("assignment to undeclared variable {key}"));
                }

                let success = crate::object::internal_methods::global::global_set_no_receiver(
                    &key.clone().into(),
                    value,
                    context,
                )?;

                if !success && context.vm.frame().code.strict {
                    return context
                        .throw_type_error(format!("cannot set non-writable property: {key}",));
                }
            }
        } else if !context.realm.environments.put_value_if_initialized(
            binding_locator.environment_index(),
            binding_locator.binding_index(),
            binding_locator.name(),
            value,
        ) {
            context.throw_reference_error(format!(
                "cannot access '{}' before initialization",
                context.interner().resolve_expect(binding_locator.name())
            ))?;
        }
        Ok(ShouldExit::False)
    }
}
