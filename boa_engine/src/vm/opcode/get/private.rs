use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `GetPrivateField` implements the Opcode Operation for `Opcode::GetPrivateField`
///
/// Operation:
///  - Get a private property by name from an object an push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPrivateField;

impl Operation for GetPrivateField {
    const NAME: &'static str = "GetPrivateField";
    const INSTRUCTION: &'static str = "INST - GetPrivateField";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let index = raw_context.vm.read::<u32>();
        let name = raw_context.vm.frame().code_block.names[index as usize].clone();
        let value = raw_context.vm.pop();
        let base_obj = value.to_object(context)?;

        let name = context
            .as_raw_context()
            .vm
            .environments
            .resolve_private_identifier(name)
            .expect("private name must be in environment");

        let result = base_obj.private_get(&name, context)?;
        context.as_raw_context_mut().vm.push(result);
        Ok(CompletionType::Normal)
    }
}
