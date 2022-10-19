use crate::{
    property::DescriptorKind,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsString, JsValue,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GetName;

impl Operation for GetName {
    const NAME: &'static str = "GetName";
    const INSTRUCTION: &'static str = "INST - GetName";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let binding_locator = context.vm.frame().code.bindings[index as usize];
        binding_locator.throw_mutate_immutable(context)?;

        let value = if binding_locator.is_global() {
            if let Some(value) = context
                .realm
                .environments
                .get_value_global_poisoned(binding_locator.name())
            {
                value
            } else {
                let key: JsString = context
                    .interner()
                    .resolve_expect(binding_locator.name().sym())
                    .into_common(false);
                match context.global_bindings_mut().get(&key) {
                    Some(desc) => match desc.kind() {
                        DescriptorKind::Data {
                            value: Some(value), ..
                        } => value.clone(),
                        DescriptorKind::Accessor { get: Some(get), .. } if !get.is_undefined() => {
                            let get = get.clone();
                            context.call(&get, &context.global_object().clone().into(), &[])?
                        }
                        _ => {
                            return context.throw_reference_error(format!(
                                "{} is not defined",
                                key.to_std_string_escaped()
                            ))
                        }
                    },
                    _ => {
                        return context.throw_reference_error(format!(
                            "{} is not defined",
                            key.to_std_string_escaped()
                        ))
                    }
                }
            }
        } else if let Some(value) = context.realm.environments.get_value_optional(
            binding_locator.environment_index(),
            binding_locator.binding_index(),
            binding_locator.name(),
        ) {
            value
        } else {
            let name = context
                .interner()
                .resolve_expect(binding_locator.name().sym())
                .to_string();
            return context.throw_reference_error(format!("{name} is not initialized"));
        };

        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GetNameOrUndefined;

impl Operation for GetNameOrUndefined {
    const NAME: &'static str = "GetNameOrUndefined";
    const INSTRUCTION: &'static str = "INST - GetNameOrUndefined";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let binding_locator = context.vm.frame().code.bindings[index as usize];
        binding_locator.throw_mutate_immutable(context)?;
        let value = if binding_locator.is_global() {
            if let Some(value) = context
                .realm
                .environments
                .get_value_global_poisoned(binding_locator.name())
            {
                value
            } else {
                let key: JsString = context
                    .interner()
                    .resolve_expect(binding_locator.name().sym())
                    .into_common(false);
                match context.global_bindings_mut().get(&key) {
                    Some(desc) => match desc.kind() {
                        DescriptorKind::Data {
                            value: Some(value), ..
                        } => value.clone(),
                        DescriptorKind::Accessor { get: Some(get), .. } if !get.is_undefined() => {
                            let get = get.clone();
                            context.call(&get, &context.global_object().clone().into(), &[])?
                        }
                        _ => JsValue::undefined(),
                    },
                    _ => JsValue::undefined(),
                }
            }
        } else if let Some(value) = context.realm.environments.get_value_optional(
            binding_locator.environment_index(),
            binding_locator.binding_index(),
            binding_locator.name(),
        ) {
            value
        } else {
            JsValue::undefined()
        };

        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}
