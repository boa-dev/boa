use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `Dup` implements the Opcode Operation for `Opcode::Dup`
///
/// Operation:
///  - Push a copy of the top value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Dup;

impl Operation for Dup {
    const NAME: &'static str = "Dup";
    const INSTRUCTION: &'static str = "INST - Dup";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        context.vm.push(value.clone());
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

/// `DupKey` implements the Opcode Operation for `Opcode::DupKey`
///
/// Operation:
/// - Duplicates the top of the property keys stack
#[derive(Debug, Clone, Copy)]
pub(crate) struct DupKey;

impl Operation for DupKey {
    const NAME: &'static str = "DupKey";
    const INSTRUCTION: &'static str = "INST - DupKey";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let key = context
            .vm
            .frame_mut()
            .keys
            .last()
            .expect("keys stack must not be empty")
            .clone();

        context.vm.frame_mut().keys.push(key);
        Ok(CompletionType::Normal)
    }
}
