use crate::{
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context,
};

/// `ToBoolean` implements the Opcode Operation for `Opcode::ToBoolean`
///
/// Operation:
///  - Pops value converts it to boolean and pushes it back.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToBoolean;

impl Operation for ToBoolean {
    const NAME: &'static str = "ToBoolean";
    const INSTRUCTION: &'static str = "INST - ToBoolean";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        context.vm.push(value.to_boolean());
        CompletionType::Normal
    }
}

/// `ToPropertyKey` implements the Opcode Operation for `Opcode::ToPropertyKey`
///
/// Operation:
///  - Call `ToPropertyKey` on the value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToPropertyKey;

impl Operation for ToPropertyKey {
    const NAME: &'static str = "ToPropertyKey";
    const INSTRUCTION: &'static str = "INST - ToPropertyKey";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        let key = ok_or_throw_completion!(value.to_property_key(context), context);
        context.vm.push(key);
        CompletionType::Normal
    }
}
