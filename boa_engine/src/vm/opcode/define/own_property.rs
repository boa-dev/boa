use crate::{
    property::PropertyDescriptor,
    vm::{ok_or_throw_completion, opcode::Operation, throw_completion, CompletionType},
    Context, JsError, JsNativeError, JsString,
};

/// `DefineOwnPropertyByName` implements the Opcode Operation for `Opcode::DefineOwnPropertyByName`
///
/// Operation:
///  - Defines a own property of an object by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineOwnPropertyByName;

impl Operation for DefineOwnPropertyByName {
    const NAME: &'static str = "DefineOwnPropertyByName";
    const INSTRUCTION: &'static str = "INST - DefineOwnPropertyByName";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let index = context.vm.read::<u32>();
        let value = context.vm.pop();
        let object = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            ok_or_throw_completion!(object.to_object(context), context)
        };
        let name = context.vm.frame().code_block.names[index as usize];
        let name = context
            .interner()
            .resolve_expect(name.sym())
            .into_common::<JsString>(false);
        ok_or_throw_completion!(
            object.__define_own_property__(
                &name.into(),
                PropertyDescriptor::builder()
                    .value(value)
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            ),
            context
        );
        CompletionType::Normal
    }
}

/// `DefineOwnPropertyByValue` implements the Opcode Operation for `Opcode::DefineOwnPropertyByValue`
///
/// Operation:
///  - Defines a own property of an object by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineOwnPropertyByValue;

impl Operation for DefineOwnPropertyByValue {
    const NAME: &'static str = "DefineOwnPropertyByValue";
    const INSTRUCTION: &'static str = "INST - DefineOwnPropertyByValue";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        let key = context.vm.pop();
        let object = context.vm.pop();
        let object = if let Some(object) = object.as_object() {
            object.clone()
        } else {
            ok_or_throw_completion!(object.to_object(context), context)
        };
        let key = ok_or_throw_completion!(key.to_property_key(context), context);
        let success = ok_or_throw_completion!(
            object.__define_own_property__(
                &key,
                PropertyDescriptor::builder()
                    .value(value)
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            ),
            context
        );
        if !success {
            throw_completion!(
                JsNativeError::typ()
                    .with_message("failed to defined own property")
                    .into(),
                JsError,
                context
            );
        }
        CompletionType::Normal
    }
}
