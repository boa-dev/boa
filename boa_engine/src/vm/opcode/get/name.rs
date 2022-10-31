use crate::{
    error::JsNativeError,
    property::DescriptorKind,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsString, JsValue,
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let binding_locator = context.vm.frame().code.bindings[index as usize];
        binding_locator.throw_mutate_immutable(context)?;

        let value = if binding_locator.is_global() {
            if let Some(value) = context
                .realm
                .environments
                .get_value_if_global_poisoned(binding_locator.name())
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
                            return Err(JsNativeError::reference()
                                .with_message(format!(
                                    "{} is not defined",
                                    key.to_std_string_escaped()
                                ))
                                .into())
                        }
                    },
                    _ => {
                        return Err(JsNativeError::reference()
                            .with_message(format!("{} is not defined", key.to_std_string_escaped()))
                            .into())
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
            return Err(JsNativeError::reference()
                .with_message(format!("{name} is not initialized"))
                .into());
        };

        context.vm.push(value);
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let binding_locator = context.vm.frame().code.bindings[index as usize];
        binding_locator.throw_mutate_immutable(context)?;
        let value = if binding_locator.is_global() {
            if let Some(value) = context
                .realm
                .environments
                .get_value_if_global_poisoned(binding_locator.name())
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
